use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};

use mime_guess::from_path;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::{error, info};
use walkdir::WalkDir;

use crate::config::Config;
use crate::handlers::thumbnail_manager::ThumbnailError;
use crate::handlers::{app_error::AppError, result_handler};
use crate::models::file_info::FileInfo;
use crate::models::{CreateFolderRequest, CreateFolderResponse, FileQuery, FilesResponse};

// Handler for GET /api/v1/files
pub async fn get_files(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileQuery>,
) -> Result<Json<FilesResponse>, AppError> {
    let path = params.path.as_deref().unwrap_or_default();
    let skip_hidden = params.skip_hidden;

    info!("Getting files from path: {}", path);

    // Construct the full path
    let full_path = PathBuf::from(&config.root_dir).join(&path);

    // Validate the path exists
    if !full_path.exists() {
        error!("Path not found: {:?}", full_path);
        return Err(AppError::NotFound(format!("Path not found: {}", path)));
    }

    // Canonicalize to resolve . and .. and get the clean absolute path
    let full_path = full_path.canonicalize().map_err(|e| {
        error!("Failed to canonicalize path {:?}: {}", full_path, e);
        AppError::NotFound(format!("Path not found: {}", path))
    })?;

    // Security: ensure the canonicalized path is still within root_dir
    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|e| {
            error!("Failed to canonicalize root directory: {}", e);
            AppError::InternalError("Invalid root directory configuration".to_string())
        })?;

    if !full_path.starts_with(&canonical_root) {
        return Err(AppError::BadRequest(
            "Invalid path: outside root directory".to_string(),
        ));
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

        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files (starting with .)
        if skip_hidden && file_name.starts_with('.') {
            continue;
        }
        // Get the absolute path of the entry
        let entry_path = entry.path();

        // Create FileInfo with absolute path and current directory context
        files.push(FileInfo::from_path(&entry_path, &full_path).map_err(|e| {
            error!("Error creating FileInfo: {}", e);
            AppError::InternalError(format!("Failed to read file info: {}", e))
        })?);
    }

    result_handler::format_result(&mut files, &params)
}

// recursivley get all videos present in path
pub async fn get_videos(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileQuery>,
) -> Result<Json<FilesResponse>, AppError> {
    let path = params.path.as_deref().unwrap_or_default();
    let skip_hidden = params.skip_hidden;

    info!("Getting videos from path: {}", path);

    // Construct the full absolute path
    let full_path = PathBuf::from(&config.root_dir).join(&path);

    // Canonicalize to resolve . and .. and get the clean absolute path
    let full_path = full_path.canonicalize().map_err(|e| {
        error!("Failed to canonicalize path {:?}: {}", full_path, e);
        AppError::NotFound(format!("Path not found: {}", path))
    })?;

    // Security: ensure the canonicalized path is still within root_dir
    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|e| {
            error!("Failed to canonicalize root directory: {}", e);
            AppError::InternalError("Invalid root directory configuration".to_string())
        })?;

    if !full_path.starts_with(&canonical_root) {
        return Err(AppError::BadRequest(
            "Invalid path: outside root directory".to_string(),
        ));
    }

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
        if skip_hidden && file_name.starts_with('.') {
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

        video_files.push(FileInfo::from_path(&file_path, &full_path).unwrap());
    }

    result_handler::format_result(&mut video_files, &params)
}

pub async fn search(
    State(config): State<Arc<Config>>,
    Query(params): Query<FileQuery>,
) -> Result<Json<FilesResponse>, AppError> {
    let path = params.path.as_deref().unwrap_or_default();
    let query = params.query.as_deref().unwrap_or_default().to_lowercase();
    let skip_hidden = params.skip_hidden;

    if query.is_empty() {
        error!("Search query is needed!");
        return Err(AppError::BadRequest(String::from("Missing search query")));
    }

    info!("Performing search in {}", path);

    // Construct the full absolute path
    let full_path = PathBuf::from(&config.root_dir).join(&path);

    // Canonicalize to resolve . and .. and get the clean absolute path
    let full_path = full_path.canonicalize().map_err(|e| {
        error!("Failed to canonicalize path {:?}: {}", full_path, e);
        AppError::NotFound(format!("Path not found: {}", path))
    })?;

    // Security: ensure the canonicalized path is still within root_dir
    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|e| {
            error!("Failed to canonicalize root directory: {}", e);
            AppError::InternalError("Invalid root directory configuration".to_string())
        })?;

    if !full_path.starts_with(&canonical_root) {
        return Err(AppError::BadRequest(
            "Invalid path: outside root directory".to_string(),
        ));
    }

    // Validate the path exists
    if !full_path.exists() {
        error!("Path not found: {:?}", full_path);
        return Err(AppError::NotFound(format!("Path not found: {}", path)));
    }

    // Must be a directory to walk
    if !full_path.is_dir() {
        return Err(AppError::BadRequest("Path is not a directory".to_string()));
    }

    let mut matching_files: Vec<FileInfo> = Vec::new();

    for entry in WalkDir::new(&full_path) {
        let entry = entry.map_err(|e| {
            error!("Error walking dir {}", e);
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
        if skip_hidden && file_name.starts_with('.') {
            continue;
        }

        if !file_name.to_lowercase().contains(&query) {
            continue;
        }

        matching_files.push(FileInfo::from_path(&file_path, &path).unwrap());
    }

    result_handler::format_result(&mut matching_files, &params)
}

pub async fn serve_file(
    State(config): State<Arc<Config>>,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file_path = file_path.trim_start_matches('/');
    let abs_path = PathBuf::from(&config.root_dir).join(file_path);

    // Security: prevent directory traversal
    if !abs_path.starts_with(&config.root_dir) {
        return Err(AppError::BadRequest("Invalid path".to_string()));
    }

    // Check if file exists and is not a directory
    if !abs_path.exists() {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    if abs_path.is_dir() {
        return Err(AppError::BadRequest("Path is a directory".to_string()));
    }

    info!("Serving file: {:?}", abs_path);

    // Open the file
    let file = File::open(&abs_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        AppError::InternalError(format!("Failed to open file: {}", e))
    })?;

    // Get file metadata for content length
    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read file metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    // Guess MIME type from file extension
    let mime_type = from_path(&abs_path).first_or_octet_stream().to_string();

    // Get filename for Content-Disposition header
    let file_name = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");

    // Create a stream from the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Build response with appropriate headers
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CONTENT_LENGTH, metadata.len().to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", file_name),
            ),
        ],
        body,
    ))
}

// Stream file (for video streaming)
pub async fn stream_file(
    State(config): State<Arc<Config>>,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file_path = file_path.trim_start_matches('/');
    let abs_path = PathBuf::from(&config.root_dir).join(file_path);

    // Security: prevent directory traversal
    if !abs_path.starts_with(&config.root_dir) {
        return Err(AppError::BadRequest("Invalid path".to_string()));
    }

    // Check if file exists and is not a directory
    if !abs_path.exists() {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    if abs_path.is_dir() {
        return Err(AppError::BadRequest("Path is a directory".to_string()));
    }

    info!("Streaming file: {:?}", abs_path);

    // Open the file
    let file = File::open(&abs_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        AppError::InternalError(format!("Failed to open file: {}", e))
    })?;

    // Get file metadata
    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read file metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    // Guess MIME type from file extension
    let mime_type = from_path(&abs_path).first_or_octet_stream().to_string();

    // Create a stream from the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Build response with streaming headers (inline, not attachment)
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CONTENT_LENGTH, metadata.len().to_string()),
            (header::ACCEPT_RANGES, "bytes".to_string()),
            (header::CACHE_CONTROL, "no-cache".to_string()),
        ],
        body,
    ))
}

pub async fn get_thumbnail(
    State(config): State<Arc<Config>>,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file_path = file_path.trim_start_matches('/');
    let abs_path = PathBuf::from(&config.root_dir).join(file_path);

    // Security: prevent directory traversal
    if !abs_path.starts_with(&config.root_dir) {
        return Err(AppError::BadRequest("Invalid path".to_string()));
    }

    let thumbnail_path =
        crate::handlers::thumbnail_manager::get_thumbnail(State(config), &abs_path)
            .await
            .map_err(|e| match e {
                ThumbnailError::InvalidInput => {
                    AppError::BadRequest("Invalid file for thumbnail generation".to_string())
                }
                ThumbnailError::InternalError(msg) => AppError::InternalError(msg),
            })?;

    // Now serve the thumbnail file
    info!("Serving thumbnail: {:?}", thumbnail_path);

    let file = File::open(&thumbnail_path).await.map_err(|e| {
        error!("Failed to open thumbnail: {}", e);
        AppError::InternalError(format!("Failed to open thumbnail: {}", e))
    })?;

    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read thumbnail metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/jpeg".to_string()),
            (header::CONTENT_LENGTH, metadata.len().to_string()),
            (header::CACHE_CONTROL, "no-cache".to_string()),
        ],
        body,
    ))
}

pub async fn create_folder(
    State(config): State<Arc<Config>>,
    Json(params): Json<CreateFolderRequest>,
) -> Result<Json<CreateFolderResponse>, AppError> {
    let path = params.path.as_deref().unwrap_or_default();
    let folder_name = params.foldername.as_deref().unwrap_or_default();

    if folder_name.is_empty() {
        return Err(AppError::NotFound(format!(
            "Folder name should not be empty"
        )));
    }

    // Construct the full path
    let full_path = PathBuf::from(&config.root_dir).join(&path);

    // Validate the path exists
    if !full_path.exists() {
        error!("Path not found: {:?}", full_path);
        return Err(AppError::NotFound(format!("Path not found: {}", path)));
    }

    // Canonicalize to resolve . and .. and get the clean absolute path
    let full_path = full_path.canonicalize().map_err(|e| {
        error!("Failed to canonicalize path {:?}: {}", full_path, e);
        AppError::NotFound(format!("Path not found: {}", path))
    })?;

    // Security: ensure the canonicalized path is still within root_dir
    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|e| {
            error!("Failed to canonicalize root directory: {}", e);
            AppError::InternalError("Invalid root directory configuration".to_string())
        })?;

    if !full_path.starts_with(&canonical_root) {
        return Err(AppError::BadRequest(
            "Invalid path: outside root directory".to_string(),
        ));
    }

    let dir_path = PathBuf::from(&full_path).join(&folder_name);

    if dir_path.exists() {
        return Err(AppError::BadRequest("Directory already exist".to_string()));
    }

    let _res = fs::create_dir_all(dir_path).map_err(|e| {
        error!("Error creating directory: {}", e);
        AppError::InternalError(format!("Failed to create directory: {}", e))
    })?;

    Ok(Json(CreateFolderResponse {
        message: String::from("Folder created successfully"),
    }))
}
