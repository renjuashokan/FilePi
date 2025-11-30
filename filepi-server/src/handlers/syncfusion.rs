use axum::{
    Json,
    body::Body,
    extract::{Form, Multipart, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use mime_guess::from_path;
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::handlers::app_error::AppError;

use syncfusion_fm_backend::{FileManagerDirectoryContent, FileManagerResponse};

pub async fn file_operations(
    State(config): State<Arc<Config>>,
    Json(args): Json<FileManagerDirectoryContent>,
) -> Result<Json<FileManagerResponse>, AppError> {
    debug!("Syncfusion FileManager action: {:?}", args);

    // call the process_file_manager_request function from syncfusion-fm-backend
    let root_dir = PathBuf::from(&config.root_dir);
    let response = syncfusion_fm_backend::process_file_manager_request(&args, &root_dir);
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct GetImageParams {
    #[serde(alias = "Path")]
    pub path: String,
}

pub async fn get_image(
    State(config): State<Arc<Config>>,
    Query(params): Query<GetImageParams>,
) -> Result<impl IntoResponse, AppError> {
    let path = params.path;
    info!("Syncfusion GetImage: {}", path);

    let relative_path = path.trim_start_matches('/');
    let root_dir = PathBuf::from(&config.root_dir);

    let full_path = syncfusion_fm_backend::validate_path(&root_dir, relative_path)
        .map_err(|_| AppError::BadRequest("Invalid path".to_string()))?;

    if !full_path.exists() {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    if !full_path.is_file() {
        return Err(AppError::BadRequest("Path is not a file".to_string()));
    }

    let file = File::open(&full_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        AppError::InternalError(format!("Failed to open file: {}", e))
    })?;

    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    let mime_type = "application/octet-stream".to_string();
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CONTENT_LENGTH, metadata.len().to_string()),
            (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
        ],
        body,
    ))
}

#[derive(Deserialize)]
pub struct DownloadForm {
    #[serde(rename = "downloadInput")]
    pub download_input: String,
}

pub async fn download(
    State(config): State<Arc<Config>>,
    Form(form): Form<DownloadForm>,
) -> Result<impl IntoResponse, AppError> {
    info!("Syncfusion Download");

    let args: FileManagerDirectoryContent =
        serde_json::from_str(&form.download_input).map_err(|e| {
            error!("Failed to parse downloadInput: {}", e);
            AppError::BadRequest("Invalid download input".to_string())
        })?;

    let path_str = args.path.as_deref().unwrap_or("/");
    let names = args.names.as_ref().ok_or(AppError::BadRequest(
        "No files specified for download".to_string(),
    ))?;

    if names.is_empty() {
        return Err(AppError::BadRequest(
            "No files specified for download".to_string(),
        ));
    }

    // For now, support single file download
    // TODO: Support multi-file download (zip)
    let file_name = &names[0];
    let relative_path = if path_str == "/" {
        file_name.clone()
    } else {
        format!("{}/{}", path_str.trim_start_matches('/'), file_name)
    };

    let root_dir = PathBuf::from(&config.root_dir);
    let full_path = syncfusion_fm_backend::validate_path(&root_dir, &relative_path)
        .map_err(|_| AppError::BadRequest("Invalid path".to_string()))?;

    if !full_path.exists() {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    if !full_path.is_file() {
        return Err(AppError::BadRequest("Path is not a file".to_string()));
    }

    let file = File::open(&full_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        AppError::InternalError(format!("Failed to open file: {}", e))
    })?;

    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    let mime_type = from_path(&full_path).first_or_octet_stream().to_string();
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let filename_header = format!("attachment; filename=\"{}\"", file_name);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CONTENT_LENGTH, metadata.len().to_string()),
            (header::CONTENT_DISPOSITION, filename_header),
        ],
        body,
    ))
}

#[derive(Deserialize)]
pub struct UploadParams {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
}

pub async fn upload(
    State(config): State<Arc<Config>>,
    Query(params): Query<UploadParams>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    info!("Syncfusion Upload2 (Streaming)");
    info!("Query params - path: {:?}, action: {:?}", params.path, params.action);

    let root_dir = PathBuf::from(&config.root_dir);
    let mut current_path = params.path.unwrap_or_else(|| String::from("/"));

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to get next field: {}", e);
        AppError::BadRequest(format!("Multipart error: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        let content_type = field.content_type().unwrap_or("").to_string();
        info!("Multipart field: name='{}', content_type='{}'", name, content_type);

        if name == "path" {
            if let Ok(val) = field.text().await {
                current_path = val.clone();
                info!("Upload path set to: '{}'", current_path);
            }
        } else if name == "action" {
             if let Ok(val) = field.text().await {
                info!("Multipart action: '{}'", val);
            }
        } else if name == "uploadFiles" {
            let file_name = field.file_name().unwrap_or("uploaded_file").to_string();
            info!("Processing file field: '{}'. Current path context: '{}'", file_name, current_path);

            let relative_path = current_path.trim_start_matches('/');
            info!("Root dir: {:?}, Relative path: '{}'", root_dir, relative_path);

            let canonical_upload_dir = syncfusion_fm_backend::validate_path(&root_dir, relative_path)
                .map_err(|_| {
                    error!("Path validation failed for relative path: '{}'", relative_path);
                    AppError::BadRequest("Invalid upload path".to_string())
                })?;
            
            info!("Canonical upload dir: {:?}", canonical_upload_dir);

            if !canonical_upload_dir.exists() {
                info!("Creating directory: {:?}", canonical_upload_dir);
                tokio::fs::create_dir_all(&canonical_upload_dir).await.map_err(|e| {
                    error!("Failed to create upload directory: {}", e);
                    AppError::InternalError(format!("Failed to create directory: {}", e))
                })?;
            }

            let file_path = canonical_upload_dir.join(&file_name);
            info!("Target file path: {:?}", file_path);

            info!("Saving file to: {:?}", file_path);

            let mut file = File::create(&file_path).await.map_err(|e| {
                error!("Failed to create file: {}", e);
                AppError::InternalError(format!("Failed to create file: {}", e))
            })?;

            let mut stream = field;
            let mut total_bytes = 0;
            while let Some(chunk) = stream.chunk().await.map_err(|e| {
                error!("Failed to read chunk: {}", e);
                AppError::InternalError(format!("Failed to read chunk: {}", e))
            })? {
                total_bytes += chunk.len();
                file.write_all(&chunk).await.map_err(|e| {
                    error!("Failed to write chunk: {}", e);
                    AppError::InternalError(format!("Failed to write chunk: {}", e))
                })?;
            }
            
            file.flush().await.map_err(|e| {
                 error!("Failed to flush file: {}", e);
                 AppError::InternalError(format!("Failed to flush file: {}", e))
            })?;
            info!("File saved successfully. Total bytes: {}", total_bytes);
        } else {
            info!("Ignoring field: name='{}'", name);
        }
    }

    Ok(StatusCode::OK)
}
