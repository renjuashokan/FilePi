use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

use crate::config::Config;
use crate::models::{ErrorResponse, FileInfo, FileQuery, FilesResponse};

// Custom error type for better error handling
pub enum AppError {
    NotFound(String),
    InternalError(String),
    BadRequest(String),
}

// Convert AppError to HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

// Handler for GET /api/v1/files
pub async fn get_files(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileQuery>,
) -> Result<Json<FilesResponse>, AppError> {
    let path = params.path.unwrap_or_default();
    let skip = params.skip.unwrap_or(0);
    let limit = params.limit.unwrap_or(25);

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

    let total = files.len();

    // Apply sorting if requested
    if let Some(sort_by) = params.sort_by {
        let order = params.order.as_deref().unwrap_or("asc");

        match sort_by.as_str() {
            "name" => {
                files.sort_by(|a, b| {
                    if order == "desc" {
                        b.name.cmp(&a.name)
                    } else {
                        a.name.cmp(&b.name)
                    }
                });
            }
            "size" => {
                files.sort_by(|a, b| {
                    if order == "desc" {
                        b.size.cmp(&a.size)
                    } else {
                        a.size.cmp(&b.size)
                    }
                });
            }
            _ => {}
        }
    }

    // Apply pagination
    let paginated_files: Vec<FileInfo> = files.into_iter().skip(skip).take(limit).collect();

    Ok(Json(FilesResponse {
        files: paginated_files,
        total,
        skip,
        limit,
    }))
}
