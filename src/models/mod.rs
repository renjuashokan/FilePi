pub mod file_info;
use crate::models::file_info::FileInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct FilesResponse {
    pub files: Vec<FileInfo>,
    pub total_files: usize,
    pub skip: usize,
    pub limit: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FileQuery {
    pub path: Option<String>,
    pub skip: Option<usize>,
    pub limit: Option<usize>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub query: Option<String>,
    #[serde(default)]
    pub skip_hidden: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
