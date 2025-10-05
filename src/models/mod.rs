use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FilesResponse {
    pub files: Vec<FileInfo>,
    pub total: usize,
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
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
