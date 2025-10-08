pub mod file_info;
use crate::models::file_info::FileInfo;
use axum_typed_multipart::{FieldData, TryFromMultipart};
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

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub path: Option<String>,
    pub foldername: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateFolderResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub message: String,
    pub filename: String,
    pub location: String,
    pub uploaded_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,
    pub skipped: bool,
}

#[derive(TryFromMultipart)]
pub struct UploadForm {
    pub location: String,
    pub user: String,
    #[form_data(limit = "10GiB")]
    pub file: FieldData<bytes::Bytes>,
    pub sha512: Option<String>,
}
