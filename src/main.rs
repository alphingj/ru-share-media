mod auth;
mod db;
mod models;
mod scanner;
mod transcode;

use axum::{
  body::Body,
  extract::{Path, State},
  http::{header::{COOKIE, CONTENT_TYPE}, HeaderValue, Method, Request, Response, StatusCode},
  response::{Html, Json},
  routing::{get, post},
  Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{info, error, warn};

use crate::auth::{generate_csrf_token, verify_password};

fn parse_cors_origins(input: String) -> AllowOrigin {
  let origins: Vec<HeaderValue> = input
    .split(',')
    .filter_map(|s| s.trim().parse::<HeaderValue>().ok())
    .collect();

  if origins.is_empty() {
    panic!("No valid CORS origins found");
  }

  AllowOrigin::list(origins)
}

async fn rate_limit_middleware(
  req: Request<Body>,
  next: axum::middleware::Next,
) -> Result<Response<Body>, StatusCode> {
  let ip = req.headers().get("X-Forwarded-For")
    .or_else(|| req.headers().get("X-Real-Ip"))
    .or_else(|| req.headers().get("X-Originating-IP"))
    .and_then(|v| v.to_str().ok())
    .unwrap_or("127.0.0.1")
    .split(',')
    .next()
    .unwrap_or("127.0.0.1")
    .trim()
    .to_string();

  const RATE_LIMIT_REQUESTS: i32 = 60;
  static RATE_LIMITER: std::sync::OnceLock<Arc<RwLock<HashMap<String, (i32, std::time::Instant)>>>> = 
    std::sync::OnceLock::new();

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

async fn security_headers(req: Request<Body>, next: axum::middleware::Next) -> Response<Body> {
  let mut response = next.run(req).await;
  let headers = response.headers_mut();
  let csp = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";
  headers.insert("Content-Security-Policy", HeaderValue::from_str(csp).unwrap());
  headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
  headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
  headers.insert("Referrer-Policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
  headers.insert("Cache-Control", HeaderValue::from_static("no-store, no-cache, must-revalidate"));
  response
}

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

async fn validate_session(pool: &SqlitePool, headers: &axum::http::HeaderMap) -> Result<(i64, bool), StatusCode> {
  let session_id = extract_session_id(headers).ok_or(StatusCode::UNAUTHORIZED)?;
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;
  
  let result: Option<(i64, i64, i64)> = sqlx::query_as(
    "SELECT s.user_id, u.is_admin, s.expires_at FROM sessions s JOIN users u ON s.user_id = u.id WHERE s.id = ? AND s.expires_at > ?"
  )
  .bind(&session_id)
  .bind(now)
  .fetch_optional(pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  match result {
    Some((user_id, is_admin, _)) => Ok((user_id, is_admin != 0)),
    None => Err(StatusCode::UNAUTHORIZED),
  }
}

#[derive(Deserialize)]
struct LoginRequest {
  username: String,
  password: String,
}

async fn login_handler(
  State(pool): State<SqlitePool>,
  Json(input): Json<LoginRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
  let user: Option<(i64, String, String)> = sqlx::query_as(
    "SELECT id, username, password_hash FROM users WHERE username = ?"
  )
  .bind(&input.username)
  .fetch_optional(&pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let (user_id, username, hash) = match user {
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
      "is_admin": false,
      "username": username
    });
    Ok((StatusCode::OK, Json(payload)))
  } else {
    Err(StatusCode::UNAUTHORIZED)
  }
}

async fn library_handler(State(pool): State<SqlitePool>, headers: axum::http::HeaderMap) -> Result<Json<Vec<models::MediaItem>>, StatusCode> {
  let (user_id, _is_admin) = validate_session(&pool, &headers).await?;
  
  let items: Vec<models::MediaItem> = sqlx::query_as(
    "SELECT f.id, f.filename, f.size_bytes, f.duration_seconds, f.width, f.height, m.title, m.year, m.poster_url, m.backdrop_url, m.media_type FROM media_files f LEFT JOIN media_metadata m ON f.id = m.media_id WHERE f.user_id = ? ORDER BY f.created_at DESC LIMIT 100"
  )
  .bind(user_id)
  .fetch_all(&pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(Json(items))
}

#[derive(Deserialize)]
struct ScanRequest {
  path: Option<String>,
}

async fn scan_handler(
  State(pool): State<SqlitePool>,
  headers: axum::http::HeaderMap,
  Json(input): Json<ScanRequest>,
) -> Result<StatusCode, StatusCode> {
  let (_user_id, is_admin) = validate_session(&pool, &headers).await?;
  if !is_admin {
    return Err(StatusCode::FORBIDDEN);
  }
  
  let path = input.path.unwrap_or_else(|| db::get_data_path());
  let count = scanner::scan_directory(&pool, &path).await.map_err(|e| {
    error!("Scan failed: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;
  info!("Scanned {} media files from {}", count, path);
  Ok(StatusCode::OK)
}

async fn stream_handler(
  State(pool): State<SqlitePool>,
  Path((media_id, _quality)): Path<(String, String)>,
  headers: axum::http::HeaderMap,
) -> Result<Response<Body>, StatusCode> {
  let (_user_id, _is_admin) = validate_session(&pool, &headers).await?;
  
  let manifest = format!("/hls/{}/master.m3u8", media_id);
  Ok(Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", HeaderValue::from_static("application/vnd.apple.mpegurl"))
    .header("Cache-Control", HeaderValue::from_static("max-age=3600"))
    .body(Body::from(manifest))
    .unwrap())
}

async fn health_handler() -> &'static str {
  "OK"
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

  let cors_origins = db::get_cors_origins();

  let app = Router::new()
    .route("/api/login", post(login_handler))
    .route("/api/library", get(library_handler))
    .route("/api/scan", post(scan_handler))
    .route("/api/health", get(health_handler))
    .route("/hls/:id/:quality", get(stream_handler))
    .nest_service("/hls", ServeDir::new(&format!("{}/hls", data_path)))
    .nest_service("/thumbnails", ServeDir::new(&format!("{}/thumbnails", data_path)))
    .nest_service("/static", ServeDir::new("static"))
    .fallback(root_handler)
    .layer(axum::middleware::from_fn(security_headers))
    .layer(
      ServiceBuilder::new()
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(
          CorsLayer::new()
            .allow_origin(parse_cors_origins(cors_origins))
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([CONTENT_TYPE, COOKIE]),
        )
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