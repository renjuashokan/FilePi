use sha2::{Digest, Sha512};
use std::fs;
use std::path::PathBuf;

pub fn compute_file_sha512(path: &PathBuf) -> Result<String, std::io::Error> {
    let contents = fs::read(path)?;
    let mut hasher = Sha512::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
