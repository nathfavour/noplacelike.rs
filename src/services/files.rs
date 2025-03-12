use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ensure_upload_folder;

/// List all files in the upload folder
pub fn list_files() -> Vec<String> {
    let upload_path = ensure_upload_folder();
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(upload_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    files
}

/// Delete a file from the upload folder
pub fn delete_file(filename: &str) -> Result<(), String> {
    let sanitized_filename = sanitize_filename::sanitize(filename);
    let file_path = ensure_upload_folder().join(sanitized_filename);
    
    if file_path.exists() {
        match fs::remove_file(file_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete file: {}", e)),
        }
    } else {
        Err("File not found".to_string())
    }
}

/// Get file path in the upload folder
pub fn get_file_path(filename: &str) -> PathBuf {
    let sanitized_filename = sanitize_filename::sanitize(filename);
    ensure_upload_folder().join(sanitized_filename)
}

/// Check if a file exists in the upload folder
pub fn file_exists(filename: &str) -> bool {
    get_file_path(filename).exists()
}
