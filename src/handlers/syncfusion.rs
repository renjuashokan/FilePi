use axum::{Json, extract::State};
use mime_guess::from_path;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

use crate::config::Config;
use crate::handlers::app_error::AppError;
use crate::models::syncfusion::{
    FileManagerDirectoryContent, FileManagerError, FileManagerItem, FileManagerResponse,
    FilePermission,
};

pub async fn file_operations(
    State(config): State<Arc<Config>>,
    Json(args): Json<FileManagerDirectoryContent>,
) -> Result<Json<FileManagerResponse>, AppError> {
    info!("Syncfusion FileManager action: {}", args.action);

    match args.action.as_str() {
        "read" => handle_read(config, args).await,
        "delete" => handle_delete(config, args).await,
        "create" => handle_create(config, args).await,
        "rename" => handle_rename(config, args).await,
        "search" => handle_search(config, args).await,
        "copy" => handle_copy(config, args).await,
        "move" => handle_move(config, args).await,
        "details" => handle_details(config, args).await,
        _ => Err(AppError::BadRequest(format!(
            "Unknown action: {}",
            args.action
        ))),
    }
}

async fn handle_read(
    config: Arc<Config>,
    args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    let mut path = args.path.as_deref().unwrap_or("");
    if path == "/" {
        path = "";
    } else if path.starts_with('/') {
        path = &path[1..];
    }
    let show_hidden = args.show_hidden_items.unwrap_or(false);

    info!("Reading directory: {}", path);

    let full_path = PathBuf::from(&config.root_dir).join(path);

    // Security checks
    let canonical_path = full_path.canonicalize().map_err(|e| {
        error!("Failed to canonicalize path: {}", e);
        AppError::NotFound("Path not found".to_string())
    })?;

    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|_| AppError::InternalError("Invalid root directory".to_string()))?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(AppError::BadRequest("Invalid path".to_string()));
    }

    if !canonical_path.is_dir() {
        return Err(AppError::BadRequest("Path is not a directory".to_string()));
    }

    // Read directory entries
    let entries = fs::read_dir(&canonical_path).map_err(|e| {
        error!("Failed to read directory: {}", e);
        AppError::InternalError("Failed to read directory".to_string())
    })?;

    let mut files = Vec::new();

    for entry in entries {
        let entry = entry
            .map_err(|_| AppError::InternalError("Failed to read directory entry".to_string()))?;

        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files if needed
        if !show_hidden && file_name.starts_with('.') {
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|_| AppError::InternalError("Failed to read file metadata".to_string()))?;

        let is_dir = metadata.is_dir();
        let file_type = if is_dir {
            "Directory".to_string()
        } else {
            get_file_extension(&file_name)
        };

        let item = FileManagerItem {
            name: file_name.clone(),
            size: metadata.len(),
            is_file: !is_dir,
            date_modified: format_system_time(&metadata.modified().ok()),
            date_created: format_system_time(&metadata.created().ok()),
            has_child: is_dir,
            filter_path: path.to_string(),
            item_type: file_type,
            icon: None,
            permission: Some(FilePermission {
                read: true,
                write: true,
                copy: true,
                download: true,
                upload: true,
                write_contents: true,
            }),
        };

        files.push(item);
    }

    // Create CWD (Current Working Directory) item
    let cwd_name = if path.is_empty() {
        canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    } else {
        path.split('/').last().unwrap_or("").to_string()
    };

    let cwd_metadata = canonical_path.metadata().ok();
    let cwd = cwd_metadata.map(|meta| FileManagerItem {
        name: cwd_name,
        size: 0,
        is_file: false,
        date_modified: format_system_time(&meta.modified().ok()),
        date_created: format_system_time(&meta.created().ok()),
        has_child: !files.is_empty(),
        filter_path: path.to_string(),
        item_type: "Directory".to_string(),
        icon: None,
        permission: Some(FilePermission {
            read: true,
            write: true,
            copy: true,
            download: true,
            upload: true,
            write_contents: true,
        }),
    });

    Ok(Json(FileManagerResponse {
        cwd,
        files,
        error: None,
    }))
}

async fn handle_create(
    config: Arc<Config>,
    args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    let path = args.path.as_deref().unwrap_or("");
    let name = args
        .name
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Folder name is required".to_string()))?;

    info!("Creating folder: {} in {}", name, path);

    let full_path = PathBuf::from(&config.root_dir).join(path).join(name);

    // Security checks
    let parent_path = PathBuf::from(&config.root_dir).join(path);
    let canonical_parent = parent_path
        .canonicalize()
        .map_err(|_| AppError::NotFound("Parent path not found".to_string()))?;

    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|_| AppError::InternalError("Invalid root directory".to_string()))?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err(AppError::BadRequest("Invalid path".to_string()));
    }

    if full_path.exists() {
        return Ok(Json(FileManagerResponse {
            cwd: None,
            files: vec![],
            error: Some(FileManagerError {
                code: "400".to_string(),
                message: "Folder already exists".to_string(),
                file_exists: Some(vec![name.to_string()]),
            }),
        }));
    }

    fs::create_dir_all(&full_path).map_err(|e| {
        error!("Failed to create directory: {}", e);
        AppError::InternalError("Failed to create directory".to_string())
    })?;

    // Return the newly created folder info
    let metadata = full_path.metadata().map_err(|_| {
        AppError::InternalError("Failed to read created folder metadata".to_string())
    })?;

    let new_folder = FileManagerItem {
        name: name.to_string(),
        size: 0,
        is_file: false,
        date_modified: format_system_time(&metadata.modified().ok()),
        date_created: format_system_time(&metadata.created().ok()),
        has_child: false,
        filter_path: path.to_string(),
        item_type: "Directory".to_string(),
        icon: None,
        permission: Some(FilePermission {
            read: true,
            write: true,
            copy: true,
            download: true,
            upload: true,
            write_contents: true,
        }),
    };

    Ok(Json(FileManagerResponse {
        cwd: None,
        files: vec![new_folder],
        error: None,
    }))
}

async fn handle_delete(
    config: Arc<Config>,
    args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    let path = args.path.as_deref().unwrap_or("");
    let names = args
        .names
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("File names are required".to_string()))?;

    info!("Deleting files: {:?} from {}", names, path);

    let mut deleted_files = Vec::new();

    for name in names {
        let full_path = PathBuf::from(&config.root_dir).join(path).join(name);

        // Security checks
        let canonical_path = full_path
            .canonicalize()
            .map_err(|_| AppError::NotFound("File not found".to_string()))?;

        let canonical_root = PathBuf::from(&config.root_dir)
            .canonicalize()
            .map_err(|_| AppError::InternalError("Invalid root directory".to_string()))?;

        if !canonical_path.starts_with(&canonical_root) {
            return Err(AppError::BadRequest("Invalid path".to_string()));
        }

        let is_dir = canonical_path.is_dir();

        // Delete file or directory
        if is_dir {
            fs::remove_dir_all(&canonical_path).map_err(|e| {
                error!("Failed to delete directory: {}", e);
                AppError::InternalError("Failed to delete directory".to_string())
            })?;
        } else {
            fs::remove_file(&canonical_path).map_err(|e| {
                error!("Failed to delete file: {}", e);
                AppError::InternalError("Failed to delete file".to_string())
            })?;
        }

        deleted_files.push(FileManagerItem {
            name: name.clone(),
            size: 0,
            is_file: !is_dir,
            date_modified: String::new(),
            date_created: String::new(),
            has_child: false,
            filter_path: path.to_string(),
            item_type: if is_dir {
                "Directory".to_string()
            } else {
                get_file_extension(name)
            },
            icon: None,
            permission: None,
        });
    }

    Ok(Json(FileManagerResponse {
        cwd: None,
        files: deleted_files,
        error: None,
    }))
}

async fn handle_rename(
    config: Arc<Config>,
    args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    let path = args.path.as_deref().unwrap_or("");
    let name = args
        .name
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("File name is required".to_string()))?;
    let new_name = args
        .new_name
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("New name is required".to_string()))?;

    info!("Renaming {} to {} in {}", name, new_name, path);

    let old_path = PathBuf::from(&config.root_dir).join(path).join(name);
    let new_path = PathBuf::from(&config.root_dir).join(path).join(new_name);

    // Security checks
    let canonical_old = old_path
        .canonicalize()
        .map_err(|_| AppError::NotFound("File not found".to_string()))?;

    let canonical_root = PathBuf::from(&config.root_dir)
        .canonicalize()
        .map_err(|_| AppError::InternalError("Invalid root directory".to_string()))?;

    if !canonical_old.starts_with(&canonical_root) {
        return Err(AppError::BadRequest("Invalid path".to_string()));
    }

    if new_path.exists() {
        return Ok(Json(FileManagerResponse {
            cwd: None,
            files: vec![],
            error: Some(FileManagerError {
                code: "400".to_string(),
                message: "File already exists".to_string(),
                file_exists: Some(vec![new_name.to_string()]),
            }),
        }));
    }

    fs::rename(&canonical_old, &new_path).map_err(|e| {
        error!("Failed to rename file: {}", e);
        AppError::InternalError("Failed to rename file".to_string())
    })?;

    let metadata = new_path
        .metadata()
        .map_err(|_| AppError::InternalError("Failed to read renamed file metadata".to_string()))?;

    let is_dir = metadata.is_dir();

    let renamed_file = FileManagerItem {
        name: new_name.to_string(),
        size: metadata.len(),
        is_file: !is_dir,
        date_modified: format_system_time(&metadata.modified().ok()),
        date_created: format_system_time(&metadata.created().ok()),
        has_child: is_dir,
        filter_path: path.to_string(),
        item_type: if is_dir {
            "Directory".to_string()
        } else {
            get_file_extension(new_name)
        },
        icon: None,
        permission: Some(FilePermission {
            read: true,
            write: true,
            copy: true,
            download: true,
            upload: true,
            write_contents: true,
        }),
    };

    Ok(Json(FileManagerResponse {
        cwd: None,
        files: vec![renamed_file],
        error: None,
    }))
}

async fn handle_search(
    _config: Arc<Config>,
    _args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    // TODO: Implement search functionality
    // For now, return empty result
    Ok(Json(FileManagerResponse {
        cwd: None,
        files: vec![],
        error: None,
    }))
}

async fn handle_copy(
    _config: Arc<Config>,
    _args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    // TODO: Implement copy functionality
    Ok(Json(FileManagerResponse {
        cwd: None,
        files: vec![],
        error: None,
    }))
}

async fn handle_move(
    _config: Arc<Config>,
    _args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    // TODO: Implement move functionality
    Ok(Json(FileManagerResponse {
        cwd: None,
        files: vec![],
        error: None,
    }))
}

async fn handle_details(
    _config: Arc<Config>,
    _args: FileManagerDirectoryContent,
) -> Result<Json<FileManagerResponse>, AppError> {
    // TODO: Implement details functionality
    Ok(Json(FileManagerResponse {
        cwd: None,
        files: vec![],
        error: None,
    }))
}

// Helper functions
fn format_system_time(time: &Option<std::time::SystemTime>) -> String {
    time.and_then(|t| {
        let datetime: chrono::DateTime<chrono::Utc> = t.into();
        Some(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
    })
    .unwrap_or_default()
}

fn _get_file_mime_type(path: &PathBuf) -> String {
    from_path(path).first_or_octet_stream().to_string()
}

fn get_file_extension(filename: &str) -> String {
    std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "file".to_string())
}
