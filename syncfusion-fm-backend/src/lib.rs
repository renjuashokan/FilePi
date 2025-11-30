pub mod models;
pub use models::*;
use std::fs;
use std::path::PathBuf;

pub fn process_file_manager_request(
    request: &FileManagerDirectoryContent,
    root_dir: &PathBuf,
) -> FileManagerResponse {
    let action = request.action.as_deref().unwrap_or("");
    match action {
        "read" => handle_read(request, root_dir),
        "create" => handle_create(request, root_dir),
        "delete" => handle_delete(request, root_dir),
        "rename" => handle_rename(request, root_dir),
        "search" => handle_search(request, root_dir),
        "copy" => handle_copy(request, root_dir),
        "move" => handle_move(request, root_dir),
        "details" => handle_details(request, root_dir),
        _ => create_error_response("400", &format!("Unknown action: {}", action)),
    }
}

fn handle_read(request: &FileManagerDirectoryContent, root_dir: &PathBuf) -> FileManagerResponse {
    let path_str = request.path.as_deref().unwrap_or("");
    let relative_path = if path_str == "/" {
        ""
    } else {
        path_str.trim_start_matches('/')
    };

    let full_path = root_dir.join(relative_path);

    // Security check
    if !is_safe_path(&full_path, root_dir) {
        return create_error_response("400", "Invalid path");
    }

    if !full_path.exists() {
        return create_error_response("404", "Path not found");
    }

    if !full_path.is_dir() {
        return create_error_response("400", "Path is not a directory");
    }

    let entries = match fs::read_dir(&full_path) {
        Ok(entries) => entries,
        Err(e) => return create_error_response("500", &format!("Failed to read directory: {}", e)),
    };

    let show_hidden = request.show_hidden_items;
    let mut files = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && file_name.starts_with('.') {
                continue;
            }

            if let Ok(metadata) = entry.metadata() {
                let is_dir = metadata.is_dir();
                let file_type = if is_dir {
                    "Directory".to_string()
                } else {
                    get_file_extension(&file_name)
                };

                files.push(FileManagerDirectoryContent {
                    name: Some(file_name),
                    size: Some(metadata.len() as i64),
                    is_file: !is_dir,
                    date_modified: metadata
                        .modified()
                        .ok()
                        .map(Into::into)
                        .or_else(|| Some(chrono::Utc::now())),
                    date_created: metadata
                        .created()
                        .ok()
                        .map(Into::into)
                        .or_else(|| Some(chrono::Utc::now())),
                    has_child: is_dir,
                    filter_path: if relative_path.is_empty() {
                        Some("/".to_string())
                    } else {
                        Some(format!("/{}/", relative_path))
                    },
                    file_type: Some(if is_dir { "".to_string() } else { file_type }),
                    permission: Some(get_default_permission()),
                    path: None,
                    action: None,
                    new_name: None,
                    names: None,
                    previous_name: None,
                    id: None,
                    filter_id: None,
                    parent_id: None,
                    target_path: None,
                    rename_files: None,
                    case_sensitive: false,
                    search_string: None,
                    show_hidden_items: false,
                    show_file_extension: false,
                    data: None,
                    target_data: None,
                });
            }
        }
    }

    let cwd_name = if relative_path.is_empty() {
        root_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    } else {
        relative_path.split('/').last().unwrap_or("").to_string()
    };

    let cwd = if let Ok(metadata) = full_path.metadata() {
        Some(FileManagerDirectoryContent {
            name: Some(cwd_name),
            size: Some(0),
            is_file: false,
            date_modified: metadata
                .modified()
                .ok()
                .map(Into::into)
                .or_else(|| Some(chrono::Utc::now())),
            date_created: metadata
                .created()
                .ok()
                .map(Into::into)
                .or_else(|| Some(chrono::Utc::now())),
            has_child: !files.is_empty(),
            filter_path: if relative_path.is_empty() {
                Some("".to_string())
            } else {
                Some(format!("/{}/", relative_path))
            },
            file_type: Some("".to_string()),
            permission: Some(get_default_permission()),
            path: None,
            action: None,
            new_name: None,
            names: None,
            previous_name: None,
            id: None,
            filter_id: None,
            parent_id: None,
            target_path: None,
            rename_files: None,
            case_sensitive: false,
            search_string: None,
            show_hidden_items: false,
            show_file_extension: false,
            data: None,
            target_data: None,
        })
    } else {
        None
    };

    FileManagerResponse {
        cwd,
        files: Some(files),
        error: None,
        details: None,
    }
}

fn handle_create(request: &FileManagerDirectoryContent, root_dir: &PathBuf) -> FileManagerResponse {
    let path_str = request.path.as_deref().unwrap_or("");
    let relative_path = if path_str == "/" {
        ""
    } else {
        path_str.trim_start_matches('/')
    };

    let name = match &request.name {
        Some(name) if !name.is_empty() => name,
        _ => return create_error_response("400", "Folder name is required"),
    };

    let full_path = root_dir.join(relative_path).join(name);

    if !is_safe_path(&full_path, root_dir) {
        return create_error_response("400", "Invalid path");
    }

    if full_path.exists() {
        return FileManagerResponse {
            cwd: None,
            files: None,
            error: Some(ErrorDetails {
                code: Some("400".to_string()),
                message: Some("Folder already exists".to_string()),
                file_exists: Some(vec![name.clone()]),
            }),
            details: None,
        };
    }

    if let Err(e) = fs::create_dir_all(&full_path) {
        return create_error_response("500", &format!("Failed to create directory: {}", e));
    }

    let metadata = match full_path.metadata() {
        Ok(m) => m,
        Err(e) => {
            return create_error_response(
                "500",
                &format!("Failed to read created folder metadata: {}", e),
            );
        }
    };

    let new_folder = FileManagerDirectoryContent {
        name: Some(name.clone()),
        size: Some(0),
        is_file: false,
        date_modified: metadata
            .modified()
            .ok()
            .map(Into::into)
            .or_else(|| Some(chrono::Utc::now())),
        date_created: metadata
            .created()
            .ok()
            .map(Into::into)
            .or_else(|| Some(chrono::Utc::now())),
        has_child: false,
        filter_path: request.path.clone(),
        file_type: Some("".to_string()),
        permission: Some(get_default_permission()),
        path: None,
        action: None,
        new_name: None,
        names: None,
        previous_name: None,
        id: None,
        filter_id: None,
        parent_id: None,
        target_path: None,
        rename_files: None,
        case_sensitive: false,
        search_string: None,
        show_hidden_items: false,
        show_file_extension: false,
        data: None,
        target_data: None,
    };

    FileManagerResponse {
        cwd: None,
        files: Some(vec![new_folder]),
        error: None,
        details: None,
    }
}

fn handle_delete(request: &FileManagerDirectoryContent, root_dir: &PathBuf) -> FileManagerResponse {
    let path_str = request.path.as_deref().unwrap_or("");
    let relative_path = if path_str == "/" {
        ""
    } else {
        path_str.trim_start_matches('/')
    };

    let names = match &request.names {
        Some(n) if !n.is_empty() => n,
        _ => return create_error_response("400", "File names are required"),
    };

    let mut deleted_files = Vec::new();

    for name in names {
        let full_path = root_dir.join(relative_path).join(name);

        if !is_safe_path(&full_path, root_dir) {
            return create_error_response("400", "Invalid path");
        }

        if !full_path.exists() {
            return create_error_response("404", "File not found");
        }

        let is_dir = full_path.is_dir();

        let result = if is_dir {
            fs::remove_dir_all(&full_path)
        } else {
            fs::remove_file(&full_path)
        };

        if let Err(e) = result {
            return create_error_response("500", &format!("Failed to delete {}: {}", name, e));
        }

        deleted_files.push(FileManagerDirectoryContent {
            name: Some(name.clone()),
            size: Some(0),
            is_file: !is_dir,
            date_modified: Some(chrono::Utc::now()), // Deleted, so maybe not relevant, but struct requires it
            date_created: Some(chrono::Utc::now()),
            has_child: false,
            filter_path: request.path.clone(),
            file_type: Some(if is_dir {
                "".to_string()
            } else {
                get_file_extension(name)
            }),
            permission: None,
            path: None,
            action: None,
            new_name: None,
            names: None,
            previous_name: None,
            id: None,
            filter_id: None,
            parent_id: None,
            target_path: None,
            rename_files: None,
            case_sensitive: false,
            search_string: None,
            show_hidden_items: false,
            show_file_extension: false,
            data: None,
            target_data: None,
        });
    }

    FileManagerResponse {
        cwd: None,
        files: Some(deleted_files),
        error: None,
        details: None,
    }
}

fn handle_rename(request: &FileManagerDirectoryContent, root_dir: &PathBuf) -> FileManagerResponse {
    let path_str = request.path.as_deref().unwrap_or("");
    let relative_path = if path_str == "/" {
        ""
    } else {
        path_str.trim_start_matches('/')
    };

    let name = match &request.name {
        Some(name) if !name.is_empty() => name,
        _ => return create_error_response("400", "File name is required"),
    };

    let new_name = match &request.new_name {
        Some(n) if !n.is_empty() => n,
        _ => return create_error_response("400", "New name is required"),
    };

    let old_path = root_dir.join(relative_path).join(name);
    let new_path = root_dir.join(relative_path).join(new_name);

    if !is_safe_path(&old_path, root_dir) || !is_safe_path(&new_path, root_dir) {
        return create_error_response("400", "Invalid path");
    }

    if !old_path.exists() {
        return create_error_response("404", "File not found");
    }

    if new_path.exists() {
        return FileManagerResponse {
            cwd: None,
            files: None,
            error: Some(ErrorDetails {
                code: Some("400".to_string()),
                message: Some("File already exists".to_string()),
                file_exists: Some(vec![new_name.clone()]),
            }),
            details: None,
        };
    }

    if let Err(e) = fs::rename(&old_path, &new_path) {
        return create_error_response("500", &format!("Failed to rename file: {}", e));
    }

    let metadata = match new_path.metadata() {
        Ok(m) => m,
        Err(e) => {
            return create_error_response(
                "500",
                &format!("Failed to read renamed file metadata: {}", e),
            );
        }
    };

    let is_dir = metadata.is_dir();

    let renamed_file = FileManagerDirectoryContent {
        name: Some(new_name.clone()),
        size: Some(metadata.len() as i64),
        is_file: !is_dir,
        date_modified: metadata
            .modified()
            .ok()
            .map(Into::into)
            .or_else(|| Some(chrono::Utc::now())),
        date_created: metadata
            .created()
            .ok()
            .map(Into::into)
            .or_else(|| Some(chrono::Utc::now())),
        has_child: is_dir,
        filter_path: request.path.clone(),
        file_type: Some(if is_dir {
            "".to_string()
        } else {
            get_file_extension(new_name)
        }),
        permission: Some(get_default_permission()),
        path: None,
        action: None,
        new_name: None,
        names: None,
        previous_name: None,
        id: None,
        filter_id: None,
        parent_id: None,
        target_path: None,
        rename_files: None,
        case_sensitive: false,
        search_string: None,
        show_hidden_items: false,
        show_file_extension: false,
        data: None,
        target_data: None,
    };

    FileManagerResponse {
        cwd: None,
        files: Some(vec![renamed_file]),
        error: None,
        details: None,
    }
}

fn handle_search(
    _request: &FileManagerDirectoryContent,
    _root_dir: &PathBuf,
) -> FileManagerResponse {
    // Placeholder
    FileManagerResponse {
        cwd: None,
        files: Some(vec![]),
        error: None,
        details: None,
    }
}

fn handle_copy(_request: &FileManagerDirectoryContent, _root_dir: &PathBuf) -> FileManagerResponse {
    // Placeholder
    FileManagerResponse {
        cwd: None,
        files: Some(vec![]),
        error: None,
        details: None,
    }
}

fn handle_move(_request: &FileManagerDirectoryContent, _root_dir: &PathBuf) -> FileManagerResponse {
    // Placeholder
    FileManagerResponse {
        cwd: None,
        files: Some(vec![]),
        error: None,
        details: None,
    }
}

fn handle_details(
    _request: &FileManagerDirectoryContent,
    _root_dir: &PathBuf,
) -> FileManagerResponse {
    // Placeholder
    FileManagerResponse {
        cwd: None,
        files: Some(vec![]),
        error: None,
        details: None,
    }
}

// Helper functions

fn create_error_response(code: &str, message: &str) -> FileManagerResponse {
    FileManagerResponse {
        cwd: None,
        files: None,
        error: Some(ErrorDetails {
            code: Some(code.to_string()),
            message: Some(message.to_string()),
            file_exists: None,
        }),
        details: None,
    }
}

pub fn validate_path(root_dir: &PathBuf, relative_path: &str) -> Result<PathBuf, String> {
    let full_path = root_dir.join(relative_path);
    if !is_safe_path(&full_path, root_dir) {
        return Err("Invalid path".to_string());
    }
    Ok(full_path)
}

fn is_safe_path(path: &PathBuf, root: &PathBuf) -> bool {
    match path.canonicalize() {
        Ok(canonical_path) => match root.canonicalize() {
            Ok(canonical_root) => canonical_path.starts_with(canonical_root),
            Err(_) => false,
        },
        Err(_) => {
            // If path doesn't exist (e.g. creating new file), check parent
            if let Some(parent) = path.parent() {
                match parent.canonicalize() {
                    Ok(canonical_parent) => match root.canonicalize() {
                        Ok(canonical_root) => canonical_parent.starts_with(canonical_root),
                        Err(_) => false,
                    },
                    Err(_) => false,
                }
            } else {
                false
            }
        }
    }
}

fn get_file_extension(filename: &str) -> String {
    std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| format!(".{}", s.to_ascii_lowercase()))
        .unwrap_or_else(|| "file".to_string())
}

fn get_default_permission() -> AccessPermission {
    AccessPermission {
        read: true,
        write: true,
        copy: true,
        download: true,
        upload: true,
        write_contents: true,
        message: String::new(),
    }
}
