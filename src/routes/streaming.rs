use actix_web::{get, web, Error, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::config::get_audio_folders;

#[derive(Debug, Serialize)]
struct AudioFilesResponse {
    files: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Deserialize)]
struct StreamQueryParams {
    file: String,
}

// Create streaming scope
pub fn stream_scope() -> Scope {
    web::scope("/stream")
        .service(list_audio)
        .service(stream_audio)
}

#[get("/list")]
async fn list_audio() -> Result<HttpResponse, Error> {
    let audio_folders = get_audio_folders();
    let mut files_by_dir = HashMap::new();
    
    for folder in audio_folders {
        if let Ok(folder_str) = folder.to_str() {
            if let Some(folder_str) = folder_str.to_owned().into() {
                let mut files = Vec::new();
                
                if let Ok(entries) = fs::read_dir(&folder) {
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
                
                files_by_dir.insert(folder_str, files);
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(AudioFilesResponse { files: files_by_dir }))
}

#[get("/play")]
async fn stream_audio(query: web::Query<StreamQueryParams>) -> Result<HttpResponse, Error> {
    let file_name = &query.file;
    let audio_folders = get_audio_folders();
    
    // Find the file in one of the audio folders
    let mut file_path = None;
    for folder in audio_folders {
        let path = folder.join(file_name);
        if path.exists() {
            file_path = Some(path);
            break;
        }
    }
    
    // If file found, stream it
    if let Some(path) = file_path {
        match read_file_content(&path) {
            Ok(content) => {
                // Get content type based on extension
                let content_type = get_content_type(&path);
                
                return Ok(HttpResponse::Ok()
                    .content_type(content_type)
                    .body(content));
            }
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                    error: format!("Failed to read file: {}", e),
                }));
            }
        }
    }
    
    Ok(HttpResponse::NotFound().json(ErrorResponse {
        error: "File not found".to_string(),
    }))
}

fn read_file_content(path: &PathBuf) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn get_content_type(path: &PathBuf) -> String {
    if let Some(extension) = path.extension() {
        match extension.to_str() {
            Some("mp3") => return "audio/mpeg".to_string(),
            Some("wav") => return "audio/wav".to_string(),
            Some("ogg") => return "audio/ogg".to_string(),
            Some("flac") => return "audio/flac".to_string(),
            _ => {}
        }
    }
    "application/octet-stream".to_string()
}
