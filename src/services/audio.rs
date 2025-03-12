use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{get_audio_folders, add_audio_folder, remove_audio_folder};

/// List audio files in all configured folders
pub fn list_audio_files() -> HashMap<String, Vec<String>> {
    let audio_folders = get_audio_folders();
    let mut files_by_dir = HashMap::new();
    
    for folder in audio_folders {
        if let Some(folder_str) = folder.to_str() {
            let mut files = Vec::new();
            
            if let Ok(entries) = fs::read_dir(&folder) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() && is_audio_file(&entry.path()) {
                                if let Some(file_name) = entry.file_name().to_str() {
                                    files.push(file_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            
            files.sort();
            files_by_dir.insert(folder_str.to_string(), files);
        }
    }
    
    files_by_dir
}

/// Check if a file is an audio file based on its extension
fn is_audio_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            let lower_ext = ext_str.to_lowercase();
            return matches!(lower_ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac");
        }
    }
    false
}

/// Add a new audio folder
pub fn add_folder(dir: &str) -> Result<(), String> {
    add_audio_folder(dir.to_string())
}

/// Remove an audio folder
pub fn remove_folder(dir: &str) -> Result<(), String> {
    remove_audio_folder(dir)
}
