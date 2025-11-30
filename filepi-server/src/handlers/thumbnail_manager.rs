use axum::extract::State;
use md5;
use mime_guess::from_path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::config::Config;

#[derive(Debug)]
pub enum ThumbnailError {
    InvalidInput,
    InternalError(String),
}

pub async fn get_thumbnail(
    State(config): State<Arc<Config>>,
    path: &PathBuf,
) -> Result<PathBuf, ThumbnailError> {
    if !path.exists() || path.is_dir() {
        return Err(ThumbnailError::InvalidInput);
    }

    let mime_type = from_path(path);
    if !mime_type
        .first_or_octet_stream()
        .essence_str()
        .starts_with("video/")
    {
        return Err(ThumbnailError::InvalidInput);
    }

    let md5_hash = get_md5_hash(&path.to_str().unwrap());
    let thumbnail_dir = &config.cache_dir.join(&md5_hash);

    if !thumbnail_dir.exists() {
        tokio::fs::create_dir_all(&thumbnail_dir)
            .await
            .map_err(|e| {
                error!("Failed to create thumbnail directory: {}", e);
                ThumbnailError::InternalError(format!(
                    "Failed to create thumbnail directory: {}",
                    e
                ))
            })?;
    }

    let thumbnail_path = thumbnail_dir.join("thumbnail.jpg");
    debug!("Thumbnail path is {:?}", thumbnail_path);

    if thumbnail_path.exists() {
        debug!("thumbnail path already exist");
        return Ok(thumbnail_path);
    }

    debug!("Generating thumbnail for {:?}", path);

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            path.to_str().unwrap(),
            "-ss",
            "00:00:05",
            "-vframes",
            "1", // Extract 1 frame
            "-vf",
            "scale=320:-1", // Scale to width 320, keep aspect ratio
            thumbnail_path.to_str().unwrap(),
            "-y",
        ])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to run FFmpeg: {}", e);
            ThumbnailError::InternalError(format!("Failed to generate thumbnail: {}", e))
        })?;
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("FFmpeg error: {}", error_msg);
        return Err(ThumbnailError::InternalError(
            "Failed to generate thumbnail with FFmpeg".to_string(),
        ));
    }

    info!("Thumbnail generated successfully");

    return Ok(thumbnail_path);
}

fn get_md5_hash(input: &str) -> String {
    let hash = md5::compute(input.as_bytes());
    format!("{:x}", hash)
}
