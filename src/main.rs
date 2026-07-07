mod auth;
mod db;
mod dlna;
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
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{info, error};

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

async fn security_headers(req: Request<Body>, next: axum::middleware::Next) -> Response<Body> {
  let mut response = next.run(req).await;
  let headers = response.headers_mut();
  let csp = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: http:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";
  headers.insert("Content-Security-Policy", HeaderValue::from_str(csp).unwrap());
  headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
  headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
  headers.insert("Referrer-Policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
  response
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

async fn library_handler(State(pool): State<SqlitePool>) -> Result<Json<Vec<models::MediaItem>>, StatusCode> {
  let items: Vec<models::MediaItem> = sqlx::query_as(
    "SELECT f.id, f.filename, f.size_bytes, f.duration_seconds, f.width, f.height, m.title, m.year, m.poster_url, m.backdrop_url, m.media_type FROM media_files f LEFT JOIN media_metadata m ON f.id = m.media_id ORDER BY f.created_at DESC LIMIT 100"
  )
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
  Json(input): Json<ScanRequest>,
) -> Result<StatusCode, StatusCode> {
  let path = input.path.unwrap_or_else(|| db::get_data_path());
  let count = scanner::scan_directory(&pool, &path).await.map_err(|e| {
    error!("Scan failed: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;
  info!("Scanned {} media files from {}", count, path);
  Ok(StatusCode::OK)
}

async fn stream_handler(
  Path(media_id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
  let manifest = format!("/hls/{}/master.m3u8", media_id);
  Ok(Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", HeaderValue::from_static("application/vnd.apple.mpegurl"))
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
  let _ = tokio::fs::create_dir_all(format!("{}/hls", data_path)).await;
  let _ = tokio::fs::create_dir_all(format!("{}/thumbnails", data_path)).await;

  let cors_origins = db::get_cors_origins();

  let app = Router::new()
    .route("/api/login", post(login_handler))
    .route("/api/library", get(library_handler))
    .route("/api/scan", post(scan_handler))
    .route("/api/health", get(health_handler))
    .route("/hls/:media_id", get(stream_handler))
    .nest_service("/hls", ServeDir::new(&format!("{}/hls", data_path)))
    .nest_service("/thumbnails", ServeDir::new(&format!("{}/thumbnails", data_path)))
    .nest_service("/static", ServeDir::new("static"))
    .fallback(root_handler)
    .layer(axum::middleware::from_fn(security_headers))
    .layer(
      ServiceBuilder::new()
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