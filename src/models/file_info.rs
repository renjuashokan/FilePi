use mime_guess::from_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub full_name: String,
    pub size: u64,
    pub is_directory: bool,
    pub created_time: Option<u128>,
    pub modified_time: Option<u128>,
    pub file_type: String, // mime type, null for directory
    pub owner: Option<String>,
    pub parent_dir: Option<String>,
    pub rel_path: Option<String>, // relative path w.r.t currrent dir
}

impl FileInfo {
    pub fn from_path<P: AsRef<Path>, T: AsRef<Path>>(
        absolute_path: P,
        current_dir: T,
    ) -> std::io::Result<Self> {
        let path = absolute_path.as_ref();
        let metadata = fs::metadata(path)?;

        // Basic info
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let full_name = String::from(path.to_str().unwrap());

        let size = match get_size(path) {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Error getting directory size: {}", e);
                return Err(e); // or handle error appropriately
            }
        };

        let is_directory = metadata.is_dir();

        // Timestamps
        let created_time = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis());

        let modified_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis());

        let file_type = from_path(&path).first_or_octet_stream().to_string();

        // Owner info (Unix/Linux only)
        let owner = get_file_owner(path);

        // Parent directory
        let parent_dir = path.parent().map(|p| p.to_string_lossy().to_string());

        // Relative path from current directory
        let rel_path = path
            .strip_prefix(&current_dir)
            .ok()
            .map(|rel| rel.to_string_lossy().to_string());

        Ok(FileInfo {
            name,
            full_name,
            size,
            is_directory,
            created_time,
            modified_time,
            file_type,
            owner,
            parent_dir,
            rel_path,
        })
    }
}

// Helper function to get file owner (Unix only)
#[cfg(unix)]
fn get_file_owner(_path: &Path) -> Option<String> {
    Some(String::from("user1"))
}

#[cfg(not(unix))]
fn get_file_owner(_path: &Path) -> Option<String> {
    // Windows doesn't have the same concept of file ownership
    // You could implement Windows-specific logic here if needed
    None
}

fn get_size<P: AsRef<Path>>(path: P) -> std::io::Result<u64> {
    let total_size = 0;
    let path = path.as_ref();

    // If it's a file, return its size
    if path.is_file() {
        let metadata = fs::metadata(path)?;
        return Ok(metadata.len());
    }

    return Ok(total_size);
    /*
    // If it's a directory, traverse recursively
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            let metadata = entry.metadata()?;
            total_size += metadata.len();
        } else if entry_path.is_dir() {
            total_size += get_size(entry_path)?;
        }
        // Skip symlinks, devices, etc.
    }

    Ok(total_size)
    */
}
