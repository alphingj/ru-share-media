use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaFile {
  pub id: String,
  pub path: String,
  pub filename: String,
  pub size_bytes: i64,
  pub mime_type: String,
  pub duration_seconds: Option<i64>,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub bitrate: Option<i64>,
  pub created_at: i64,
  pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaMetadata {
  pub media_id: String,
  pub tmdb_id: Option<i64>,
  pub imdb_id: Option<String>,
  pub title: String,
  pub original_title: Option<String>,
  pub year: Option<i32>,
  pub synopsis: Option<String>,
  pub poster_url: Option<String>,
  pub backdrop_url: Option<String>,
  pub media_type: Option<String>,
  pub season: Option<i32>,
  pub episode: Option<i32>,
  pub content_rating: Option<String>,
  pub release_date: Option<String>,
  pub last_synced: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaItem {
  pub id: String,
  pub filename: String,
  pub size_bytes: i64,
  pub duration_seconds: Option<i64>,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub title: Option<String>,
  pub year: Option<i32>,
  pub poster_url: Option<String>,
  pub backdrop_url: Option<String>,
  pub media_type: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
  pub id: i64,
  pub username: String,
  pub password_hash: String,
  pub is_admin: i64,
  pub preferences: Option<String>,
  pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WatchHistory {
  pub user_id: i64,
  pub media_id: String,
  pub position_seconds: i64,
  pub last_watched: i64,
  pub played: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlaylist {
  pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistItem {
  pub media_id: String,
  pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
  pub total_files: i64,
  pub total_size_bytes: i64,
  pub total_users: i64,
}