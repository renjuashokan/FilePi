use axum::Json;

use crate::handlers::app_error::AppError;
use crate::models::file_info::FileInfo;

use crate::models::{FileQuery, FilesResponse};

pub fn format_result(
    files: &mut Vec<FileInfo>,
    params: &FileQuery,
) -> Result<Json<FilesResponse>, AppError> {
    let skip = params.skip.unwrap_or(0);
    let limit = params.limit.unwrap_or(25);
    let total = files.len();

    // Sorting modifies the vector → that's why it's &mut
    let sort_by = params.sort_by.as_deref();
    let order = params.order.as_deref().unwrap_or("asc");
    let is_desc = order == "desc";

    if let Some(sort_field) = sort_by {
        // Validate sort field
        let valid_fields = ["name", "size", "modified_time", "created_time", "file_type"];
        if !valid_fields.contains(&sort_field) {
            return Err(AppError::BadRequest(format!(
                "Invalid sort field: {}",
                sort_field
            )));
        }

        // Sort with directories always first
        files.sort_by(|a, b| {
            // ALWAYS sort directories first, then files - for ALL sort criteria
            if a.is_directory != b.is_directory {
                return b.is_directory.cmp(&a.is_directory);
            }

            // If both are directories or both are files, then sort by the specified field
            match sort_field {
                "name" => {
                    if is_desc {
                        b.name.cmp(&a.name)
                    } else {
                        a.name.cmp(&b.name)
                    }
                }
                "size" => {
                    if is_desc {
                        b.size.cmp(&a.size)
                    } else {
                        a.size.cmp(&b.size)
                    }
                }
                "modified_time" => {
                    if is_desc {
                        b.modified_time.cmp(&a.modified_time)
                    } else {
                        a.modified_time.cmp(&b.modified_time)
                    }
                }
                "created_time" => {
                    if is_desc {
                        b.created_time.cmp(&a.created_time)
                    } else {
                        a.created_time.cmp(&b.created_time)
                    }
                }
                "file_type" => {
                    if is_desc {
                        b.file_type.cmp(&a.file_type)
                    } else {
                        a.file_type.cmp(&b.file_type)
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });
    } else {
        // Default sorting: directories first, then by name (ascending)
        files.sort_by(|a, b| {
            // Always sort directories first, then files
            if a.is_directory != b.is_directory {
                return b.is_directory.cmp(&a.is_directory);
            }
            // If both are directories or both are files, sort by name
            a.name.cmp(&b.name)
        });
    }

    // Pagination
    let paginated_files: Vec<FileInfo> = files.iter().skip(skip).take(limit).cloned().collect();

    Ok(Json(FilesResponse {
        files: paginated_files,
        total_files: total,
        skip,
        limit,
    }))
}
