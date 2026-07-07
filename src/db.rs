use sqlx::SqlitePool;

pub fn get_data_path() -> String {
  std::env::var("RU_SHARE_MEDIA_PATH").unwrap_or_else(|_| {
    let cwd = std::env::current_dir().unwrap_or_default();
    format!("{}/media", cwd.display())
  })
}

pub fn get_listen_addr() -> String {
  std::env::var("RU_SHARE_LISTEN").unwrap_or_else(|_| "0.0.0.0:8080".to_string())
}

pub fn get_cors_origins() -> String {
  std::env::var("RU_SHARE_CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

pub fn get_db_path() -> String {
  let data_path = get_data_path();
  format!("{}/media.db", data_path)
}

pub async fn init_pool() -> Result<SqlitePool, sqlx::Error> {
  let db_path = get_db_path();
  let path = std::path::PathBuf::from(&db_path);
  if let Some(parent) = path.parent() {
    let _ = tokio::fs::create_dir_all(parent).await;
  }
  let pool = sqlx::sqlite::SqlitePoolOptions::new()
    .max_connections(10)
    .connect(&format!("sqlite://{}?mode=rwc", db_path))
    .await?;
  Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
  let migration = include_str!("../migrations/001_initial.sql");
  sqlx::query(migration).execute(pool).await?;
  Ok(())
}

pub async fn create_admin_user(pool: &SqlitePool) -> Result<(), sqlx::Error> {
  let admin_user = std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
  let admin_pass = std::env::var("ADMIN_PASS").unwrap_or_else(|_| "changeme".to_string());
  
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;
  
  let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
    .bind(&admin_user)
    .fetch_optional(pool)
    .await?;
  
  if exists.is_none() {
    let password_hash = crate::auth::hash_password(&admin_pass).unwrap_or_default();
    
    sqlx::query("INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, 1, ?)")
      .bind(&admin_user)
      .bind(&password_hash)
      .bind(now)
      .execute(pool)
      .await?;
  }
  Ok(())
}