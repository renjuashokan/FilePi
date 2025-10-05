use axum::{
    Json,
    extract::{Query, State},
};

use mime_guess::from_path;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};
use walkdir::WalkDir;

use crate::config::Config;
use crate::handlers::{app_error::AppError, result_handler};
use crate::models::{FileInfo, FileQuery, FilesResponse};

// Handler for GET /api/v1/files
pub async fn get_files(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileQuery>,
) -> Result<Json<FilesResponse>, AppError> {
    let path = params.path.as_deref().unwrap_or_default();

    info!("Getting files from path: {}", path);

    // Construct the full path
    let full_path = PathBuf::from(&config.root_dir).join(&path);

    // Validate the path exists
    if !full_path.exists() {
        error!("Path not found: {:?}", full_path);
        return Err(AppError::NotFound(format!("Path not found: {}", path)));
    }

    // Check if it's a directory
    if !full_path.is_dir() {
        return Err(AppError::BadRequest("Path is not a directory".to_string()));
    }

    // Read directory contents
    let entries = fs::read_dir(&full_path).map_err(|e| {
        error!("Error reading directory: {}", e);
        AppError::InternalError(format!("Failed to read directory: {}", e))
    })?;

    // Collect file information
    let mut files: Vec<FileInfo> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| {
            error!("Error reading entry: {}", e);
            AppError::InternalError(format!("Failed to read entry: {}", e))
        })?;

        let metadata = entry.metadata().map_err(|e| {
            error!("Error reading metadata: {}", e);
            AppError::InternalError(format!("Failed to read metadata: {}", e))
        })?;

        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files (starting with .)
        if file_name.starts_with('.') {
            continue;
        }

        let relative_path = if path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", path, file_name)
        };

        let modified = metadata.modified().ok().and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs().to_string())
        });

        files.push(FileInfo {
            name: file_name,
            path: relative_path,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        });
    }

    result_handler::format_result(&mut files, &params)
}

// recursivley get all videos present in path
pub async fn get_videos(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileQuery>,
) -> Result<Json<FilesResponse>, AppError> {
    let path = params.path.as_deref().unwrap_or_default();

    info!("Getting videos from path: {}", path);

    // Construct the full absolute path
    let full_path = PathBuf::from(&config.root_dir).join(&path);

    // Validate the path exists
    if !full_path.exists() {
        error!("Path not found: {:?}", full_path);
        return Err(AppError::NotFound(format!("Path not found: {}", path)));
    }

    // Must be a directory to walk
    if !full_path.is_dir() {
        return Err(AppError::BadRequest("Path is not a directory".to_string()));
    }

    let mut video_files: Vec<FileInfo> = Vec::new();

    // Walk the directory recursively
    for entry in WalkDir::new(&full_path) {
        let entry = entry.map_err(|e| {
            error!("Error walking directory: {}", e);
            AppError::InternalError(format!("Failed to traverse directory: {}", e))
        })?;

        let metadata = entry.metadata().map_err(|e| {
            error!("Error reading metadata for {:?}: {}", entry.path(), e);
            AppError::InternalError(format!("Failed to read file metadata: {}", e))
        })?;

        // Skip directories
        if metadata.is_dir() {
            continue;
        }

        let file_path = entry.path();
        let file_name = match file_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue, // Skip if no filename (shouldn't happen normally)
        };

        // Skip hidden files
        if file_name.starts_with('.') {
            continue;
        }

        // Guess MIME type from file extension
        let mime_type = from_path(file_path);
        if !mime_type
            .first_or_octet_stream()
            .essence_str()
            .starts_with("video/")
        {
            continue;
        }

        // Compute relative path from the requested `path` (not from root_dir)
        // We want: requested_path + relative_part
        let relative_part = match file_path.strip_prefix(&full_path) {
            Ok(rel) => rel,
            Err(e) => {
                error!("Failed to compute relative path: {}", e);
                continue;
            }
        };

        let relative_path = if relative_part.as_os_str().is_empty() {
            file_name.clone()
        } else {
            // Use forward slashes for URL-friendly paths
            let rel_str = relative_part.to_string_lossy().replace('\\', "/");
            if path.is_empty() {
                rel_str
            } else {
                format!("{}/{}", path, rel_str)
            }
        };

        let modified = metadata.modified().ok().and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs().to_string())
        });

        video_files.push(FileInfo {
            name: file_name,
            path: relative_path,
            is_dir: false,
            size: metadata.len(),
            modified,
        });
    }

    result_handler::format_result(&mut video_files, &params)
}
