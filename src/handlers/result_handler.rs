use axum::Json;

use crate::handlers::app_error::AppError;

use crate::models::{FileInfo, FileQuery, FilesResponse};

pub fn format_result(
    files: &mut Vec<FileInfo>, // still mutable in case you sort
    params: &FileQuery,
) -> Result<Json<FilesResponse>, AppError> {
    let skip = params.skip.unwrap_or(0);
    let limit = params.limit.unwrap_or(25);
    let total = files.len();

    // Sorting modifies the vector → that's why it's &mut
    if let Some(sort_by) = params.sort_by.as_deref() {
        let order = params.order.as_deref().unwrap_or("asc");
        match sort_by {
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

    // Now just read from it
    let paginated_files: Vec<FileInfo> = files.iter().skip(skip).take(limit).cloned().collect();

    Ok(Json(FilesResponse {
        files: paginated_files,
        total,
        skip,
        limit,
    }))
}
