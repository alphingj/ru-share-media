// Media Scanner Module
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::process::Command;

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "aac", "ogg", "m4a"];
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

fn get_media_type(filename: &str) -> Option<&'static str> {
  let ext = PathBuf::from(filename)
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| e.to_lowercase());
  
  if let Some(ext) = ext {
    if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
      Some("video")
    } else if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
      Some("audio")
    } else if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
      Some("image")
    } else {
      None
    }
  } else {
    None
  }
}

pub async fn scan_directory(pool: &SqlitePool, media_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
  let mut count = 0;
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)?
    .as_secs() as i64;

  let mut stack = vec![PathBuf::from(media_path)];

  while let Some(current_dir) = stack.pop() {
    if current_dir.is_dir() {
      if let Ok(mut entries) = tokio::fs::read_dir(&current_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
          let path = entry.path();
          if path.is_dir() {
            stack.push(path);
          } else if let Some(media_type) = get_media_type(&path.to_string_lossy()) {
            let id = uuid::Uuid::new_v4().to_string();
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let size_bytes = entry.metadata().await?.len() as i64;

            let (duration, width, height, bitrate) = probe_with_ffprobe(&path).await;

            sqlx::query(
              "INSERT OR REPLACE INTO media_files (id, path, filename, size_bytes, mime_type, duration_seconds, width, height, bitrate, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&id)
            .bind(&path.to_string_lossy().to_string())
            .bind(&filename)
            .bind(size_bytes)
            .bind(media_type)
            .bind(duration)
            .bind(width)
            .bind(height)
            .bind(bitrate)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

            count += 1;
          }
        }
      }
    }
  }

  Ok(count)
}

async fn probe_with_ffprobe(path: &PathBuf) -> (Option<i64>, Option<i32>, Option<i32>, Option<i64>) {
  let output = Command::new("ffprobe")
    .args(&[
      "-v", "quiet",
      "-print_format", "json",
      "-show_format",
      "-show_streams",
      path.to_str().unwrap(),
    ])
    .output();

  if let Ok(output) = output {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
      let format = json.get("format");
      if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
        let duration = format
          .and_then(|f| f.get("duration"))
          .and_then(|d| d.as_str())
          .and_then(|d| d.parse::<f64>().ok())
          .map(|d| d as i64);
        
        let video_stream = streams.iter().find(|s| s.get("codec_type").and_then(|c| c.as_str()) == Some("video"));
        let (width, height, bitrate) = video_stream.map(|stream| {
          let w = stream.get("width").and_then(|w| w.as_i64()).map(|w| w as i32);
          let h = stream.get("height").and_then(|h| h.as_i64()).map(|h| h as i32);
          let b = format
            .and_then(|f| f.get("bit_rate"))
            .and_then(|b| b.as_str())
            .and_then(|b| b.parse::<i64>().ok());
          (w, h, b)
        }).unwrap_or((None, None, None));
        
        return (duration, width, height, bitrate);
      }
    }
  }

  (None, None, None, None)
}

pub async fn generate_thumbnail(path: &PathBuf, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  let _ = Command::new("ffmpeg")
    .args(&[
      "-y",
      "-i", path.to_str().unwrap(),
      "-ss", "00:00:10",
      "-vframes", "1",
      "-vf", "scale=320:-1",
      output_path,
    ])
    .output();
  Ok(())
}