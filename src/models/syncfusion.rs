use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileManagerDirectoryContent {
    pub action: String,

    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub show_hidden_items: Option<bool>,

    #[serde(default)]
    pub names: Option<Vec<String>>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub new_name: Option<String>,

    #[serde(default)]
    pub target_path: Option<String>,

    #[serde(default)]
    pub rename_files: Option<Vec<String>>,

    #[serde(default)]
    pub search_string: Option<String>,

    #[serde(default)]
    pub case_sensitive: Option<bool>,

    #[serde(default)]
    pub data: Option<Vec<FileManagerItemInput>>,

    #[serde(default)]
    pub target_data: Option<FileManagerItemInput>,

    // Additional fields that Syncfusion might send
    #[serde(default)]
    pub show_file_extension: Option<bool>,

    #[serde(default)]
    pub custom_data: Option<serde_json::Value>,

    #[serde(default)]
    pub id: Option<String>,
}

// Input model - what Syncfusion sends to us (more permissive)
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileManagerItemInput {
    pub name: String,

    #[serde(default)]
    pub is_file: bool,

    #[serde(default)]
    pub size: u64,

    #[serde(default)]
    pub date_modified: Option<String>,

    #[serde(default)]
    pub date_created: Option<String>,

    #[serde(default)]
    pub has_child: bool,

    #[serde(default)]
    pub filter_path: Option<String>,

    #[serde(default, rename = "type")]
    pub item_type: Option<String>,

    #[serde(default)]
    pub icon: Option<String>,

    #[serde(default)]
    pub permission: Option<FilePermissionInput>,

    // Additional fields that Syncfusion sends
    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub action: Option<String>,

    #[serde(default)]
    pub new_name: Option<String>,

    #[serde(default)]
    pub names: Option<Vec<String>>,

    #[serde(default)]
    pub previous_name: Option<String>,

    #[serde(default)]
    pub id: Option<String>,

    #[serde(default)]
    pub filter_id: Option<String>,

    #[serde(default)]
    pub parent_id: Option<String>,

    #[serde(default)]
    pub target_path: Option<String>,

    #[serde(default)]
    pub rename_files: Option<Vec<String>>,

    #[serde(default)]
    pub case_sensitive: Option<bool>,

    #[serde(default)]
    pub search_string: Option<String>,

    #[serde(default)]
    pub show_hidden_items: Option<bool>,

    #[serde(default)]
    pub data: Option<serde_json::Value>,

    #[serde(default)]
    pub target_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilePermissionInput {
    #[serde(default)]
    pub read: bool,

    #[serde(default)]
    pub write: bool,

    #[serde(default)]
    pub copy: bool,

    #[serde(default)]
    pub download: bool,

    #[serde(default)]
    pub upload: bool,

    #[serde(default)]
    pub write_contents: bool,

    #[serde(default)]
    pub message: Option<String>,
}

// Output model - what we send back to Syncfusion (strict)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileManagerItem {
    pub name: String,

    #[serde(rename = "isFile")]
    pub is_file: bool,

    pub size: u64,

    #[serde(rename = "dateModified")]
    pub date_modified: String,

    #[serde(rename = "dateCreated")]
    pub date_created: String,

    #[serde(rename = "hasChild")]
    pub has_child: bool,

    #[serde(rename = "filterPath")]
    pub filter_path: String,

    #[serde(rename = "type")]
    pub item_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "permission")]
    pub permission: Option<FilePermission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePermission {
    pub read: bool,
    pub write: bool,
    pub copy: bool,
    pub download: bool,
    pub upload: bool,
    #[serde(rename = "writeContents")]
    pub write_contents: bool,
}

#[derive(Serialize)]
pub struct FileManagerError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_exists: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct FileManagerResponse {
    pub cwd: Option<FileManagerItem>,
    pub files: Vec<FileManagerItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FileManagerError>,
}
