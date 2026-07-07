// Transcoder Module
use std::process::Command;
use std::path::PathBuf;

const QUALITIES: &[(&str, i32, i32)] = &[
  ("1080p", 1920, 1080),
  ("720p", 1280, 720),
  ("480p", 854, 480),
  ("360p", 640, 360),
];

pub async fn transcode_to_hls(
  input_path: &PathBuf,
  output_dir: &PathBuf,
  quality: &str,
) -> Result<String, Box<dyn std::error::Error>> {
  let quality_cfg = QUALITIES.iter().find(|(q, _, _)| *q == quality);
  if quality_cfg.is_none() {
    return Err("Invalid quality".into());
  }

  let (_name, width, _height) = quality_cfg.unwrap();
  
  let playlist_path = output_dir.join(format!("{}.m3u8", quality));
  let segment_pattern = output_dir.join(format!("{}%03d.ts", quality));

  let status = Command::new("ffmpeg")
    .args(&[
      "-y",
      "-i", input_path.to_str().unwrap(),
      "-preset", "ultrafast",
      "-g", "30",
      "-keyint_min", "30",
      "-sc_threshold", "0",
      "-vf", &format!("scale={}:-2", width),
      "-c:v", "libx264",
      "-b:v", "2M",
      "-maxrate", "2M",
      "-bufsize", "4M",
      "-hls_time", "10",
      "-hls_list_size", "0",
      "-hls_segment_filename", segment_pattern.to_str().unwrap(),
      playlist_path.to_str().unwrap(),
    ])
    .status()?;

  if status.success() {
    Ok(format!("/{}.m3u8", quality))
  } else {
    Err("Transcoding failed".into())
  }
}