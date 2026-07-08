mod auth;
mod db;
mod models;
mod scanner;

use axum::{
  body::Body,
  extract::{Path, State},
  http::{header::COOKIE, HeaderValue, Method, Request, Response, StatusCode},
  response::{Html, Json},
  routing::{delete, get, post},
  Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tracing::{info, error, warn};

use crate::auth::{extract_csrf_token, generate_csrf_token, validate_csrf_token, verify_password};

// ===================== CORS with CIDR subnet support =====================

struct CorsConfig {
  exact: Vec<String>,
  subnets: Vec<(u32, u32)>, // (network, mask) host order, IPv4
}

static CORS_CONFIG: OnceLock<Arc<CorsConfig>> = OnceLock::new();

fn parse_cidr(cidr: &str) -> Option<(u32, u32)> {
  let (ip_part, prefix_part) = cidr.split_once('/')?;
  let prefix: u32 = prefix_part.parse().ok()?;
  let octets: Vec<&str> = ip_part.split('.').collect();
  if octets.len() != 4 {
    return None;
  }
  let mut addr: u32 = 0;
  for o in &octets {
    let v: u8 = o.parse().ok()?;
    addr = (addr << 8) | (v as u32);
  }
  if prefix > 32 {
    return None;
  }
  let mask = if prefix == 0 { 0u32 } else { (!0u32) << (32 - prefix) };
  Some((addr & mask, mask))
}

fn build_cors_config(input: &str) -> CorsConfig {
  let mut exact = Vec::new();
  let mut subnets = Vec::new();
  for part in input.split(',') {
    let p = part.trim();
    if p.is_empty() {
      continue;
    }
    if p.contains('/') {
      if let Some(c) = parse_cidr(p) {
        subnets.push(c);
      }
      continue;
    }
    exact.push(p.to_string());
  }
  CorsConfig { exact, subnets }
}

fn origin_allowed(cfg: &CorsConfig, origin: &str) -> bool {
  if cfg.exact.iter().any(|e| e == origin) {
    return true;
  }
  if let Some(host) = origin.split("://").nth(1) {
    let host_only = host.split(':').next().unwrap_or(host);
    if let Ok(IpAddr::V4(ipv4)) = host_only.parse::<IpAddr>() {
      let addr = u32::from(ipv4);
      return cfg.subnets.iter().any(|(net, mask)| (addr & mask) == *net);
    }
  }
  false
}

fn apply_cors_to(cfg: &CorsConfig, origin: &str, headers: &mut axum::http::HeaderMap) {
  if origin_allowed(cfg, origin) {
    if let Ok(hv) = HeaderValue::from_str(origin) {
      headers.insert("Access-Control-Allow-Origin", hv);
      headers.insert("Access-Control-Allow-Credentials", HeaderValue::from_static("true"));
      headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
      );
      headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Content-Type, Cookie, X-CSRF-Token"),
      );
    }
  }
}

async fn cors_middleware(
  req: Request<Body>,
  next: axum::middleware::Next,
) -> Response<Body> {
  let origin = req.headers().get("origin").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
  let is_preflight = req.method() == Method::OPTIONS;

  if is_preflight {
    let mut resp = Response::new(Body::empty());
    if let (Some(cfg), Some(o)) = (CORS_CONFIG.get(), &origin) {
      apply_cors_to(cfg, o, resp.headers_mut());
    }
    return resp;
  }

  let mut response = next.run(req).await;
  if let (Some(cfg), Some(o)) = (CORS_CONFIG.get(), &origin) {
    apply_cors_to(cfg, o, response.headers_mut());
  }
  response
}

// ===================== Rate limiting =====================

const RATE_LIMIT_REQUESTS: i32 = 60;

async fn rate_limit_middleware(
  req: Request<Body>,
  next: axum::middleware::Next,
) -> Result<Response<Body>, StatusCode> {
  let ip = req.headers().get("X-Forwarded-For")
    .or_else(|| req.headers().get("X-Real-Ip"))
    .and_then(|v| v.to_str().ok())
    .unwrap_or("127.0.0.1")
    .split(',')
    .next()
    .unwrap_or("127.0.0.1")
    .trim()
    .to_string();

  static RATE_LIMITER: OnceLock<Arc<RwLock<HashMap<String, (i32, std::time::Instant)>>>> =
    OnceLock::new();

  let limiter = RATE_LIMITER.get_or_init(|| Arc::new(RwLock::new(HashMap::new())));
  {
    let mut limiter = limiter.write().await;
    let now = std::time::Instant::now();
    let entry = limiter.entry(ip.clone()).or_insert((RATE_LIMIT_REQUESTS, now));
    const REFILL_SECS: u64 = 60;
    if now.duration_since(entry.1).as_secs() > REFILL_SECS {
      entry.0 = RATE_LIMIT_REQUESTS;
      entry.1 = now;
    }
    if entry.0 > 0 {
      entry.0 -= 1;
    } else {
      warn!("Rate limit exceeded for IP: {}", ip);
      return Err(StatusCode::TOO_MANY_REQUESTS);
    }
  }

  Ok(next.run(req).await)
}

// ===================== Security headers =====================

async fn security_headers(req: Request<Body>, next: axum::middleware::Next) -> Response<Body> {
  let mut response = next.run(req).await;
  let headers = response.headers_mut();
  let csp = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";
  headers.insert("Content-Security-Policy", HeaderValue::from_str(csp).unwrap());
  headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
  headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
  headers.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));
  headers.insert("Referrer-Policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
  headers.insert("Permissions-Policy", HeaderValue::from_static("camera=(), microphone=(), geolocation=()"));
  headers.insert("Cache-Control", HeaderValue::from_static("no-store, no-cache, must-revalidate"));
  response
}

// ===================== Session helpers =====================

fn extract_session_id(headers: &axum::http::HeaderMap) -> Option<String> {
  headers.get(COOKIE).and_then(|v| v.to_str().ok()).and_then(|cookie| {
    cookie.split(';').find_map(|pair| {
      let trimmed = pair.trim();
      if trimmed.starts_with("session_id=") {
        Some(trimmed.trim_start_matches("session_id=").to_string())
      } else {
        None
      }
    })
  })
}

/// Validate session and return (user_id, is_admin, csrf_token).
async fn validate_session_full(pool: &SqlitePool, headers: &axum::http::HeaderMap)
  -> Result<(i64, bool, String), StatusCode> {
  let session_id = extract_session_id(headers).ok_or(StatusCode::UNAUTHORIZED)?;
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  let result: Option<(i64, i64, String, i64)> = sqlx::query_as(
    "SELECT s.user_id, u.is_admin, s.csrf_token, s.expires_at FROM sessions s JOIN users u ON s.user_id = u.id WHERE s.id = ? AND s.expires_at > ?"
  )
  .bind(&session_id)
  .bind(now)
  .fetch_optional(pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  match result {
    Some((user_id, is_admin, csrf, _)) => Ok((user_id, is_admin != 0, csrf)),
    None => Err(StatusCode::UNAUTHORIZED),
  }
}

async fn validate_session(pool: &SqlitePool, headers: &axum::http::HeaderMap)
  -> Result<(i64, bool), StatusCode> {
  let (uid, admin, _) = validate_session_full(pool, headers).await?;
  Ok((uid, admin))
}

/// Validate session + CSRF token (for state-changing requests).
async fn validate_session_csrf(pool: &SqlitePool, headers: &axum::http::HeaderMap)
  -> Result<(i64, bool), StatusCode> {
  let (uid, admin, csrf) = validate_session_full(pool, headers).await?;
  let provided = extract_csrf_token(headers).ok_or(StatusCode::FORBIDDEN)?;
  if !validate_csrf_token(&provided, &csrf) {
    return Err(StatusCode::FORBIDDEN);
  }
  Ok((uid, admin))
}

// ===================== Handlers =====================

#[derive(Deserialize)]
struct LoginRequest {
  username: String,
  password: String,
}

async fn login_handler(
  State(pool): State<SqlitePool>,
  Json(input): Json<LoginRequest>,
) -> Result<Response<Body>, StatusCode> {
  let user: Option<(i64, String, String, i64)> = sqlx::query_as(
    "SELECT id, username, password_hash, is_admin FROM users WHERE username = ?"
  )
  .bind(&input.username)
  .fetch_optional(&pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let (user_id, username, hash, is_admin) = match user {
    Some(u) => u,
    None => return Err(StatusCode::UNAUTHORIZED),
  };

  if verify_password(&input.password, &hash).unwrap_or(false) {
    let session_id = uuid::Uuid::new_v4().to_string();
    let csrf_token = generate_csrf_token();
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64
      + 604800;

    sqlx::query("INSERT INTO sessions (id, user_id, csrf_token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
      .bind(&session_id)
      .bind(user_id)
      .bind(&csrf_token)
      .bind(now)
      .bind(now)
      .execute(&pool)
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payload = serde_json::json!({
      "session_id": session_id,
      "csrf_token": csrf_token,
      "is_admin": is_admin != 0,
      "username": username
    });

    let cookie_value = format!("session_id={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}", 
      session_id, 604800);

    Ok(Response::builder()
      .status(StatusCode::OK)
      .header("Set-Cookie", cookie_value)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&payload).unwrap()))
      .unwrap())
  } else {
    Err(StatusCode::UNAUTHORIZED)
  }
}

// Library: all media (shared family library)
async fn library_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
) -> Result<Json<Vec<models::MediaItem>>, StatusCode> {
  let (_user_id, _is_admin) = validate_session(&pool, &headers).await?;

  let items: Vec<models::MediaItem> = sqlx::query_as(
    "SELECT f.id, f.filename, f.size_bytes, f.duration_seconds, f.width, f.height, m.title, m.year, m.poster_url, m.backdrop_url, m.media_type FROM media_files f LEFT JOIN media_metadata m ON f.id = m.media_id ORDER BY f.created_at DESC LIMIT 200"
  )
  .fetch_all(&pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(Json(items))
}

// Continue watching (real watch history)
async fn continue_watching_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
  let (user_id, _is_admin) = validate_session(&pool, &headers).await?;

  let rows: Vec<(String, String, i64, Option<String>)> = sqlx::query_as(
    "SELECT w.media_id, f.filename, w.position_seconds, m.title FROM watch_history w \
     JOIN media_files f ON w.media_id = f.id \
     LEFT JOIN media_metadata m ON w.media_id = m.media_id \
     WHERE w.user_id = ? AND w.played = 0 ORDER BY w.last_watched DESC LIMIT 20"
  )
  .bind(user_id)
  .fetch_all(&pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let items = rows.into_iter().map(|(id, filename, pos, title)| {
    serde_json::json!({
      "id": id,
      "filename": filename,
      "title": title,
      "position_seconds": pos
    })
  }).collect();

  Ok(Json(items))
}

#[derive(Deserialize)]
struct ScanRequest {
  path: Option<String>,
}

// Global scan mutex to prevent CPU/disk exhaustion
static SCAN_LOCK: OnceLock<Arc<tokio::sync::Mutex<()>>> = OnceLock::new();

async fn scan_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
  Json(input): Json<ScanRequest>,
) -> Result<StatusCode, StatusCode> {
  let (_user_id, is_admin) = validate_session_csrf(&pool, &headers).await?;
  if !is_admin {
    return Err(StatusCode::FORBIDDEN);
  }

  let lock = SCAN_LOCK.get_or_init(|| Arc::new(tokio::sync::Mutex::new(())));
  let _guard = match lock.try_lock() {
    Ok(g) => g,
    Err(_) => return Err(StatusCode::CONFLICT),
  };

  let requested_path = input.path.unwrap_or_else(|| db::get_data_path());
  let canonical = std::path::PathBuf::from(&requested_path);
  let canonical = match std::path::absolute(&canonical) {
    Ok(p) => p,
    Err(_) => return Err(StatusCode::BAD_REQUEST),
  };

  let media_root = match std::path::absolute(&std::path::PathBuf::from(&db::get_data_path())) {
    Ok(p) => p,
    Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
  };

  if !canonical.starts_with(&media_root) {
    return Err(StatusCode::BAD_REQUEST);
  }

  let path = canonical.to_string_lossy().to_string();
  let count = scanner::scan_directory(&pool, &path).await.map_err(|e| {
    error!("Scan failed: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;
  info!("Scanned {} media files from {}", count, path);
  Ok(StatusCode::OK)
}

async fn health_handler() -> &'static str {
  "OK"
}

async fn logout_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
  if let Some(session_id) = extract_session_id(&headers) {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
      .bind(&session_id)
      .execute(&pool)
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  }
  Ok(StatusCode::NO_CONTENT)
}

// ===================== Admin User Management =====================

#[derive(Deserialize)]
struct CreateUserRequest {
  username: String,
  password: String,
}

async fn create_user_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
  Json(input): Json<CreateUserRequest>,
) -> Result<StatusCode, StatusCode> {
  let (_user_id, is_admin) = validate_session_csrf(&pool, &headers).await?;
  if !is_admin {
    return Err(StatusCode::FORBIDDEN);
  }

  if input.username.is_empty() || input.password.is_empty() || input.password.len() < 6 {
    return Err(StatusCode::BAD_REQUEST);
  }

  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
    .bind(&input.username)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  if exists.is_some() {
    return Err(StatusCode::CONFLICT);
  }

  let password_hash = match crate::auth::hash_password(&input.password) {
    Ok(h) => h,
    Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
  };

  sqlx::query("INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, 0, ?)")
    .bind(&input.username)
    .bind(&password_hash)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(StatusCode::CREATED)
}

async fn list_users_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
  let (_user_id, is_admin) = validate_session(&pool, &headers).await?;
  if !is_admin {
    return Err(StatusCode::FORBIDDEN);
  }

  let users: Vec<(i64, String, i64, i64)> = sqlx::query_as(
    "SELECT id, username, is_admin, created_at FROM users ORDER BY username"
  )
  .fetch_all(&pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let items = users.into_iter().map(|u| {
    serde_json::json!({
      "id": u.0,
      "username": u.1,
      "is_admin": u.2 != 0,
      "created_at": u.3
    })
  }).collect();

  Ok(Json(items))
}

async fn delete_user_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
  Path(user_id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
  let (admin_user_id, is_admin) = validate_session_csrf(&pool, &headers).await?;
  if !is_admin {
    return Err(StatusCode::FORBIDDEN);
  }

  if user_id == admin_user_id {
    return Err(StatusCode::BAD_REQUEST);
  }

  let is_target_admin: Option<i64> = sqlx::query_scalar("SELECT is_admin FROM users WHERE id = ?")
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  if is_target_admin.map(|a| a != 0).unwrap_or(true) {
    return Err(StatusCode::FORBIDDEN);
  }

  sqlx::query("DELETE FROM users WHERE id = ?")
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let pool = db::init_pool().await.expect("Failed to init DB");
  db::run_migrations(&pool).await.expect("Failed to run migrations");
  db::create_admin_user(&pool).await.expect("Failed to create admin");

  let data_path = db::get_data_path();
  let _ = tokio::fs::create_dir_all(&format!("{}/hls", data_path)).await;
  let _ = tokio::fs::create_dir_all(&format!("{}/thumbnails", data_path)).await;

  // Build CORS config (supports exact origins and CIDR subnets) and store globally
  let cors_input = db::get_cors_origins();
  CORS_CONFIG.get_or_init(|| Arc::new(build_cors_config(&cors_input)));

  let app = Router::new()
    .route("/api/login", post(login_handler))
    .route("/api/logout", get(logout_handler))
    .route("/api/library", get(library_handler))
    .route("/api/continue-watching", get(continue_watching_handler))
    .route("/api/scan", post(scan_handler))
    .route("/api/health", get(health_handler))
    .route("/api/admin/users", get(list_users_handler).post(create_user_handler))
    .route("/api/admin/users/:id", delete(delete_user_handler))
    .nest_service("/hls", ServeDir::new(&format!("{}/hls", data_path)))
    .nest_service("/thumbnails", ServeDir::new(&format!("{}/thumbnails", data_path)))
    .fallback(root_handler)
    .layer(axum::middleware::from_fn(security_headers))
    .layer(
      ServiceBuilder::new()
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::middleware::from_fn(cors_middleware))
        .into_inner(),
    )
    .with_state(pool);

  let addr: SocketAddr = db::get_listen_addr()
    .parse()
    .expect("Invalid listen address");

  info!("Media server listening on {}", addr);
  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn root_handler(_uri: axum::http::Uri) -> Html<&'static str> {
  Html(include_str!("../static/index.html"))
}
