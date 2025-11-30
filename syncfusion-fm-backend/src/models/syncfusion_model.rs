use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessPermission {
    pub copy: bool,
    pub download: bool,
    pub write: bool,
    pub write_contents: bool,
    pub read: bool,
    pub upload: bool,
    pub message: String,
}

impl Default for AccessPermission {
    fn default() -> Self {
        Self {
            copy: true,
            download: true,
            write: true,
            write_contents: true,
            read: true,
            upload: true,
            message: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileManagerDirectoryContent {
    pub path: Option<String>,
    pub action: Option<String>,
    pub new_name: Option<String>,
    pub names: Option<Vec<String>>,
    pub name: Option<String>,
    pub size: Option<i64>,
    pub previous_name: Option<String>,
    pub date_modified: Option<DateTime<Utc>>,
    pub date_created: Option<DateTime<Utc>>,
    #[serde(default)]
    pub has_child: bool,
    #[serde(default)]
    pub is_file: bool,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub id: Option<String>,
    pub filter_path: Option<String>,
    pub filter_id: Option<String>,
    pub parent_id: Option<String>,
    pub target_path: Option<String>,
    pub rename_files: Option<Vec<String>>,
    // UploadFiles is omitted as it's usually handled via multipart forms, not JSON body
    #[serde(default)]
    pub case_sensitive: bool,
    pub search_string: Option<String>,
    #[serde(default)]
    pub show_hidden_items: bool,
    #[serde(default)]
    pub show_file_extension: bool,
    pub data: Option<Vec<FileManagerDirectoryContent>>,
    pub target_data: Option<Box<FileManagerDirectoryContent>>,
    pub permission: Option<AccessPermission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileDetails {
    pub name: Option<String>,
    pub location: Option<String>,
    pub is_file: bool,
    pub size: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub multiple_files: bool,
    pub permission: Option<AccessPermission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetails {
    pub code: Option<String>,
    pub message: Option<String>,
    pub file_exists: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileManagerResponse {
    #[serde(rename = "cwd")]
    pub cwd: Option<FileManagerDirectoryContent>,
    pub files: Option<Vec<FileManagerDirectoryContent>>,
    pub error: Option<ErrorDetails>,
    pub details: Option<FileDetails>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessDetails {
    pub role: Option<String>,
    pub access_rules: Option<Vec<AccessRule>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessRule {
    pub copy: Option<Permission>,
    pub download: Option<Permission>,
    pub write: Option<Permission>,
    pub path: Option<String>,
    pub read: Option<Permission>,
    pub role: Option<String>,
    pub write_contents: Option<Permission>,
    pub upload: Option<Permission>,
    pub is_file: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Permission {
    Allow,
    Deny,
}
