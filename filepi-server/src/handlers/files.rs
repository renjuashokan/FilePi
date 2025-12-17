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
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;
use tracing::{debug, error, info};
use walkdir::WalkDir;

use crate::config::Config;
use crate::handlers::hash_utilities::{compute_file_sha1_streaming, compute_file_sha512_streaming};
use crate::handlers::thumbnail_manager::ThumbnailError;
use crate::handlers::{app_error::AppError, result_handler};
use crate::models::file_info::FileInfo;
use crate::models::{CreateFolderRequest, CreateFolderResponse, FileQuery, FilesResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServeFileParams {
    pub inline: Option<bool>,
}

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
        files.push(
            FileInfo::from_path(&entry_path, &full_path, &config.root_dir).map_err(|e| {
                error!("Error creating FileInfo: {}", e);
                AppError::InternalError(format!("Failed to read file info: {}", e))
            })?,
        );
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

        video_files.push(
            FileInfo::from_path(&file_path, &full_path, &config.root_dir).map_err(|e| {
                error!("Error creating FileInfo: {}", e);
                AppError::InternalError(format!("Failed to read file info: {}", e))
            })?,
        );
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

        matching_files.push(
            FileInfo::from_path(&file_path, &full_path, &config.root_dir).map_err(|e| {
                error!("Error creating FileInfo: {}", e);
                AppError::InternalError(format!("Failed to read file info: {}", e))
            })?,
        );
    }

    result_handler::format_result(&mut matching_files, &params)
}

pub async fn serve_file(
    State(config): State<Arc<Config>>,
    Path(file_path): Path<String>,
    Query(params): Query<ServeFileParams>,
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
                if params.inline.unwrap_or(false) {
                    format!("inline; filename=\"{}\"", file_name)
                } else {
                    format!("attachment; filename=\"{}\"", file_name)
                },
            ),
        ],
        body,
    ))
}

// Stream file (for video streaming with Range request support)
pub async fn stream_file(
    State(config): State<Arc<Config>>,
    Path(file_path): Path<String>,
    headers: axum::http::HeaderMap,
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
    let mut file = File::open(&abs_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        AppError::InternalError(format!("Failed to open file: {}", e))
    })?;

    // Get file metadata
    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read file metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    let file_size = metadata.len();

    // Guess MIME type from file extension
    let mime_type = from_path(&abs_path).first_or_octet_stream().to_string();

    // Parse Range header if present
    let range_header = headers.get(header::RANGE);

    if let Some(range_value) = range_header {
        // Parse the Range header (format: "bytes=start-end")
        let range_str = range_value.to_str().unwrap_or("");

        if let Some(byte_range) = parse_range_header(range_str, file_size) {
            let (start, end) = byte_range;
            let content_length = end - start + 1;

            info!(
                "Range request: bytes {}-{}/{} (length: {})",
                start, end, file_size, content_length
            );

            // Seek to the start position
            use tokio::io::AsyncSeekExt;
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|e| {
                    error!("Failed to seek file: {}", e);
                    AppError::InternalError(format!("Failed to seek file: {}", e))
                })?;

            // Create a limited stream that only reads the requested range
            let limited_file = file.take(content_length);
            let stream = ReaderStream::new(limited_file);
            let body = Body::from_stream(stream);

            // Return 206 Partial Content with Content-Range header
            return Ok((
                StatusCode::PARTIAL_CONTENT,
                [
                    (header::CONTENT_TYPE, mime_type),
                    (header::CONTENT_LENGTH, content_length.to_string()),
                    (
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", start, end, file_size),
                    ),
                    (header::ACCEPT_RANGES, "bytes".to_string()),
                    (header::CACHE_CONTROL, "no-cache".to_string()),
                ],
                body,
            ));
        }
    }

    // No range header or invalid range - return full file
    info!("Full file request: {} bytes", file_size);

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Build response with streaming headers (inline, not attachment)
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CONTENT_LENGTH, file_size.to_string()),
            (header::ACCEPT_RANGES, "bytes".to_string()),
            (header::CACHE_CONTROL, "no-cache".to_string()),
            (header::CONTENT_DISPOSITION, "inline".to_string()),
        ],
        body,
    ))
}

// Helper function to parse Range header
// Returns (start, end) inclusive byte positions
fn parse_range_header(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    // Expected format: "bytes=start-end" or "bytes=start-" or "bytes=-suffix"
    let range_str = range_str.trim();

    if !range_str.starts_with("bytes=") {
        return None;
    }

    let range_part = &range_str[6..]; // Skip "bytes="

    if let Some((start_str, end_str)) = range_part.split_once('-') {
        if start_str.is_empty() {
            // Suffix range: "-500" means last 500 bytes
            if let Ok(suffix) = end_str.parse::<u64>() {
                let start = file_size.saturating_sub(suffix);
                return Some((start, file_size - 1));
            }
        } else if end_str.is_empty() {
            // Open-ended range: "500-" means from byte 500 to end
            if let Ok(start) = start_str.parse::<u64>() {
                if start < file_size {
                    return Some((start, file_size - 1));
                }
            }
        } else {
            // Full range: "500-999"
            if let (Ok(start), Ok(end)) = (start_str.parse::<u64>(), end_str.parse::<u64>()) {
                if start <= end && start < file_size {
                    let end = end.min(file_size - 1);
                    return Some((start, end));
                }
            }
        }
    }

    None
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
    Query(params): Query<CreateFolderRequest>,
) -> Result<Json<CreateFolderResponse>, AppError> {
    info!("=== Create folder request received ===");

    let path = params.path.as_deref().unwrap_or_default();
    let folder_name = params.foldername.as_deref().unwrap_or_default();

    info!(
        "Create folder - path: '{}', folder_name: '{}'",
        path, folder_name
    );

    if folder_name.is_empty() {
        error!("Folder name is empty");
        return Err(AppError::NotFound(format!(
            "Folder name should not be empty"
        )));
    }
    debug!("✓ Folder name validated");

    // Construct the full path
    let full_path = PathBuf::from(&config.root_dir).join(&path);
    debug!("Full path constructed: {:?}", full_path);

    // Validate the path exists
    if !full_path.exists() {
        error!("Path not found: {:?}", full_path);
        return Err(AppError::NotFound(format!("Path not found: {}", path)));
    }
    debug!("✓ Path exists");

    // Canonicalize to resolve . and .. and get the clean absolute path
    let full_path = full_path.canonicalize().map_err(|e| {
        error!("Failed to canonicalize path {:?}: {}", full_path, e);
        AppError::NotFound(format!("Path not found: {}", path))
    })?;
    debug!("✓ Canonical path: {:?}", full_path);

    // Security: ensure the canonicalized path is still within root_dir
    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|e| {
            error!("Failed to canonicalize root directory: {}", e);
            AppError::InternalError("Invalid root directory configuration".to_string())
        })?;

    if !full_path.starts_with(&canonical_root) {
        error!(
            "Security violation: path {:?} is outside root {:?}",
            full_path, canonical_root
        );
        return Err(AppError::BadRequest(
            "Invalid path: outside root directory".to_string(),
        ));
    }
    debug!("✓ Security check passed");

    let dir_path = PathBuf::from(&full_path).join(&folder_name);
    debug!("Target directory path: {:?}", dir_path);

    if dir_path.exists() {
        error!("Directory already exists: {:?}", dir_path);
        return Err(AppError::BadRequest("Directory already exist".to_string()));
    }
    debug!("✓ Directory does not exist, creating...");

    let _res = fs::create_dir_all(&dir_path).map_err(|e| {
        error!("Error creating directory: {}", e);
        AppError::InternalError(format!("Failed to create directory: {}", e))
    })?;

    info!("✓✓✓ Folder created successfully: {:?}", dir_path);

    Ok(Json(CreateFolderResponse {
        message: String::from("Folder created successfully"),
    }))
}

// Streaming upload handler - processes file chunks without loading entire file into memory
pub async fn upload_file(
    State(config): State<Arc<Config>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<crate::models::UploadResponse>, AppError> {
    info!("=== Starting streaming file upload process ===");

    let mut location = String::new();
    let mut user = String::new();
    let mut client_sha512: Option<String> = None;
    let mut client_sha1: Option<String> = None;
    let mut filename = String::from("unnamed");
    let mut file_path: Option<PathBuf> = None;
    let mut total_bytes = 0u64;
    let mut field_count = 0;

    debug!("Beginning multipart field iteration");

    // Process multipart fields
    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        AppError::BadRequest(format!("Invalid multipart data: {}", e))
    })? {
        field_count += 1;
        let field_name = field.name().unwrap_or("").to_string();
        debug!("Processing field #{}: '{}'", field_count, field_name);

        match field_name.as_str() {
            "location" => {
                debug!("Reading 'location' field...");
                location = field.text().await.map_err(|e| {
                    error!("Failed to read location field: {}", e);
                    AppError::BadRequest("Invalid location field".to_string())
                })?;
                debug!("✓ Upload location: '{}'", location);
            }
            "user" => {
                debug!("Reading 'user' field...");
                user = field.text().await.map_err(|e| {
                    error!("Failed to read user field: {}", e);
                    AppError::BadRequest("Invalid user field".to_string())
                })?;
                debug!("✓ Upload user: '{}'", user);
            }
            "sha512" => {
                debug!("Reading 'sha512' field...");
                client_sha512 = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| {
                            error!("Failed to read sha512 field: {}", e);
                            AppError::BadRequest("Invalid sha512 field".to_string())
                        })?
                        .trim()
                        .to_lowercase(),
                );
                debug!("✓ Client provided SHA-512 hash");
            }
            "sha1" => {
                debug!("Reading 'sha1' field...");
                client_sha1 = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| {
                            error!("Failed to read sha1 field: {}", e);
                            AppError::BadRequest("Invalid sha1 field".to_string())
                        })?
                        .trim()
                        .to_lowercase(),
                );
                debug!("✓ Client provided SHA-1 hash");
            }
            "file" => {
                debug!("Processing 'file' field...");

                // Validate required fields before processing file
                if location.is_empty() || user.is_empty() {
                    error!(
                        "Missing required fields - location: '{}', user: '{}'",
                        location, user
                    );
                    return Err(AppError::BadRequest(
                        "Missing required fields: location or user must be provided before file"
                            .to_string(),
                    ));
                }
                debug!("✓ Required fields validated");

                // Get filename from field metadata
                let raw_filename = field.file_name().unwrap_or("unnamed");
                debug!("Raw filename from client: '{}'", raw_filename);

                filename = raw_filename.replace(['/', '\\', '\0'], "_");
                debug!("✓ Sanitized filename: '{}'", filename);

                // Construct the full path for upload location
                let upload_dir = PathBuf::from(&config.root_dir).join(&location);
                debug!("Upload directory path: {:?}", upload_dir);

                // Canonicalize and validate the upload directory
                let upload_dir = if upload_dir.exists() {
                    debug!("Upload directory exists, canonicalizing...");
                    upload_dir.canonicalize().map_err(|e| {
                        error!("Failed to canonicalize upload path: {}", e);
                        AppError::BadRequest("Invalid upload location".to_string())
                    })?
                } else {
                    debug!("Upload directory doesn't exist, creating...");
                    fs::create_dir_all(&upload_dir).map_err(|e| {
                        error!("Failed to create upload directory: {}", e);
                        AppError::InternalError(format!("Failed to create directory: {}", e))
                    })?;
                    debug!("✓ Directory created");
                    upload_dir.canonicalize().map_err(|e| {
                        error!("Failed to canonicalize upload path: {}", e);
                        AppError::BadRequest("Invalid upload location".to_string())
                    })?
                };
                debug!("✓ Canonical upload directory: {:?}", upload_dir);

                // Security: ensure the upload path is within root_dir
                let canonical_root =
                    PathBuf::from(&config.root_dir)
                        .canonicalize()
                        .map_err(|e| {
                            error!("Failed to canonicalize root directory: {}", e);
                            AppError::InternalError(
                                "Invalid root directory configuration".to_string(),
                            )
                        })?;

                if !upload_dir.starts_with(&canonical_root) {
                    error!(
                        "Security violation: upload path {:?} is outside root {:?}",
                        upload_dir, canonical_root
                    );
                    return Err(AppError::BadRequest(
                        "Invalid upload path: outside root directory".to_string(),
                    ));
                }
                debug!("✓ Security check passed");

                // Full path for the file
                let target_path = upload_dir.join(&filename);
                debug!("Target file path: {:?}", target_path);

                // Check if file already exists and hash is provided for deduplication
                if target_path.exists() {
                    debug!("File already exists at target path");

                    // Try SHA-512 first (more secure)
                    if let Some(ref client_hash) = client_sha512 {
                        info!("Checking SHA-512 hash for deduplication...");

                        // Compute SHA-512 hash of existing file using async streaming
                        let existing_hash = compute_file_sha512_streaming(&target_path)
                            .await
                            .map_err(|e| {
                                error!("Failed to compute SHA-512 hash of existing file: {}", e);
                                AppError::InternalError(format!(
                                    "Failed to compute file hash: {}",
                                    e
                                ))
                            })?;

                        info!(
                            "Client SHA-512: {}..., Existing file SHA-512: {}...",
                            &client_hash[..16.min(client_hash.len())],
                            &existing_hash[..16]
                        );

                        // If hashes match, skip upload
                        if client_hash == &existing_hash {
                            info!("✓ SHA-512 match - skipping upload for file: {}", filename);

                            let relative_path = target_path
                                .strip_prefix(&canonical_root)
                                .unwrap_or(&target_path)
                                .to_string_lossy()
                                .to_string();

                            return Ok(Json(crate::models::UploadResponse {
                                message:
                                    "File already exists with identical content, upload skipped"
                                        .to_string(),
                                filename,
                                location: relative_path,
                                uploaded_by: user,
                                skipped: true,
                                sha512: Some(existing_hash),
                                sha1: None,
                            }));
                        } else {
                            info!("SHA-512 mismatch - file will be replaced");
                        }
                    } else if let Some(ref client_hash) = client_sha1 {
                        info!("Checking SHA-1 hash for deduplication...");

                        // Compute SHA-1 hash of existing file using async streaming
                        let existing_hash = compute_file_sha1_streaming(&target_path)
                            .await
                            .map_err(|e| {
                                error!("Failed to compute SHA-1 hash of existing file: {}", e);
                                AppError::InternalError(format!(
                                    "Failed to compute file hash: {}",
                                    e
                                ))
                            })?;

                        info!(
                            "Client SHA-1: {}..., Existing file SHA-1: {}...",
                            &client_hash[..16.min(client_hash.len())],
                            &existing_hash[..16]
                        );

                        // If hashes match, skip upload
                        if client_hash == &existing_hash {
                            info!("✓ SHA-1 match - skipping upload for file: {}", filename);

                            let relative_path = target_path
                                .strip_prefix(&canonical_root)
                                .unwrap_or(&target_path)
                                .to_string_lossy()
                                .to_string();

                            return Ok(Json(crate::models::UploadResponse {
                                message:
                                    "File already exists with identical content, upload skipped"
                                        .to_string(),
                                filename,
                                location: relative_path,
                                uploaded_by: user,
                                skipped: true,
                                sha512: None,
                                sha1: Some(existing_hash),
                            }));
                        } else {
                            info!("SHA-1 mismatch - file will be replaced");
                        }
                    } else {
                        info!("No hash provided - file will be replaced");
                    }
                } else {
                    debug!("File does not exist, will create new file");
                }

                debug!("Creating output file for writing...");
                // Create file for writing (async)
                let mut output_file = tokio::fs::File::create(&target_path).await.map_err(|e| {
                    error!("Failed to create file: {}", e);
                    AppError::InternalError(format!("Failed to create file: {}", e))
                })?;
                debug!("✓ Output file created successfully");

                // Stream chunks directly to file
                debug!("Starting chunk streaming...");
                use tokio::io::AsyncWriteExt;
                let mut chunk_count = 0;

                while let Some(chunk) = field.chunk().await.map_err(|e| {
                    error!("Failed to read file chunk #{}: {}", chunk_count + 1, e);
                    AppError::InternalError(format!("Failed to read file chunk: {}", e))
                })? {
                    chunk_count += 1;
                    let chunk_size = chunk.len();
                    total_bytes += chunk_size as u64;

                    if chunk_count % 100 == 0 {
                        debug!(
                            "Processing chunk #{}, size: {} bytes, total: {} bytes",
                            chunk_count, chunk_size, total_bytes
                        );
                    }

                    output_file.write_all(&chunk).await.map_err(|e| {
                        error!(
                            "Failed to write chunk #{} ({} bytes): {}",
                            chunk_count, chunk_size, e
                        );
                        AppError::InternalError(format!("Failed to write file chunk: {}", e))
                    })?;
                }
                info!(
                    "✓ Finished streaming {} chunks, total {} bytes",
                    chunk_count, total_bytes
                );

                // Ensure all data is written to disk
                debug!("Flushing file to disk...");
                output_file.flush().await.map_err(|e| {
                    error!("Failed to flush file: {}", e);
                    AppError::InternalError(format!("Failed to flush file: {}", e))
                })?;
                debug!("✓ File flushed successfully");

                file_path = Some(target_path);
                info!(
                    "✓✓✓ File streamed successfully: '{}' ({} bytes, {} chunks)",
                    filename, total_bytes, chunk_count
                );

                // Break after processing file - it's the last field and stream is consumed
                debug!("Breaking from field loop after file processing");
                break;
            }
            _ => {
                // Skip unknown fields
                debug!("Skipping unknown field: '{}'", field_name);
            }
        }
    }

    debug!("Finished processing {} multipart fields", field_count);

    // Validate that we received a file
    let file_path = file_path.ok_or_else(|| {
        error!(
            "No file field found in multipart data after processing {} fields",
            field_count
        );
        AppError::BadRequest("No file provided in upload".to_string())
    })?;
    debug!("✓ File path validated: {:?}", file_path);

    // Compute SHA-512 hash of newly uploaded file
    // info!("Computing SHA-512 hash of uploaded file...");
    // let new_file_hash = compute_file_sha512(&file_path).map_err(|e| {
    //     error!("Failed to compute SHA-512 hash of uploaded file: {}", e);
    //     AppError::InternalError(format!("Failed to compute file hash: {}", e))
    // })?;
    // info!("✓ New file SHA-512: {}...", &new_file_hash[..16]);

    // Get the relative path from root_dir
    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|e| {
            error!("Failed to canonicalize root directory: {}", e);
            AppError::InternalError("Invalid root directory configuration".to_string())
        })?;

    let relative_path = file_path
        .strip_prefix(&canonical_root)
        .unwrap_or(&file_path)
        .to_string_lossy()
        .to_string();

    info!(
        "=== Upload completed successfully: '{}' at '{}' ===",
        filename, relative_path
    );

    Ok(Json(crate::models::UploadResponse {
        message: "File uploaded successfully".to_string(),
        filename,
        location: relative_path,
        uploaded_by: user,
        skipped: false,
        sha512: None,
        sha1: None,
    }))
}
