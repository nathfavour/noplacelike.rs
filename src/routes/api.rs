use actix_files::NamedFile;
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Result, Scope, ResponseError};
use arboard::Clipboard;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path, sync::Mutex, fmt};

use crate::config::ensure_upload_folder;

#[derive(Debug, Serialize, Deserialize)]
struct ClipboardRequest {
    text: String,
}

#[derive(Debug, Serialize)]
struct FileListResponse {
    files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    filename: Option<String>,
    error: Option<String>,
}

// Static clipboard storage
pub type ClipboardData = Mutex<String>;

// Create API scope
pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(get_clipboard)
        .service(post_clipboard)
        .service(list_files)
        .service(upload_file)
        .service(download_file)
}

#[get("/clipboard")]
async fn get_clipboard(clipboard_data: web::Data<ClipboardData>) -> Result<HttpResponse> {
    let text = clipboard_data.lock().unwrap().clone();
    
    Ok(HttpResponse::Ok().json(ClipboardRequest { text }))
}

#[post("/clipboard")]
async fn post_clipboard(
    clipboard_data: web::Data<ClipboardData>,
    req: web::Json<ClipboardRequest>,
) -> Result<HttpResponse> {
    let text = req.text.clone();
    
    // Update in-memory clipboard
    {
        let mut clipboard = clipboard_data.lock().unwrap();
        *clipboard = text.clone();
    }
    
    // Try to update system clipboard if available
    match Clipboard::new() {
        Ok(mut clipboard) => {
            if let Err(e) = clipboard.set_text(text) {
                eprintln!("Failed to update system clipboard: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to access system clipboard: {}", e);
        }
    }
    
    Ok(HttpResponse::Ok().json(StatusResponse {
        status: "success".to_string(),
        filename: None,
        error: None,
    }))
}

#[get("/files")]
async fn list_files() -> Result<HttpResponse> {
    let upload_path = ensure_upload_folder();
    
    let mut files = Vec::new();
    
    match fs::read_dir(upload_path) {
        Ok(entries) => {
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
        Err(e) => {
            eprintln!("Failed to read upload directory: {}", e);
        }
    }
    
    Ok(HttpResponse::Ok().json(FileListResponse { files }))
}

#[post("/files")]
async fn upload_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let upload_path = ensure_upload_folder();
    
    // Process multipart upload
    while let Some(field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap_or("unnamed_file");
        let sanitized_filename = sanitize_filename::sanitize(filename);
        
        let file_path = upload_path.join(&sanitized_filename);
        
        // Save file
        if let Err(e) = save_file(field, file_path).await {
            return Ok(HttpResponse::InternalServerError().json(StatusResponse {
                status: "error".to_string(),
                filename: Some(sanitized_filename),
                error: Some(format!("Failed to save file: {}", e)),
            }));
        }
        
        return Ok(HttpResponse::Ok().json(StatusResponse {
            status: "success".to_string(),
            filename: Some(sanitized_filename),
            error: None,
        }));
    }
    
    Ok(HttpResponse::BadRequest().json(StatusResponse {
        status: "error".to_string(),
        filename: None,
        error: Some("No file provided".to_string()),
    }))
}

#[get("/files/{filename}")]
async fn download_file(req: HttpRequest, filename: web::Path<String>) -> Result<HttpResponse, Error> {
    let sanitized_filename = sanitize_filename::sanitize(filename.as_str());
    let file_path = ensure_upload_folder().join(&sanitized_filename);
    
    match NamedFile::open(&file_path) {
        Ok(file) => {
            let file_name = file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let disposition = actix_web::http::header::ContentDisposition {
                disposition: actix_web::http::header::DispositionType::Attachment,
                parameters: vec![
                    actix_web::http::header::DispositionParam::Filename(file_name)
                ],
            };
            
            Ok(file
                .use_last_modified(true)
                .set_content_disposition(disposition)
                .into_response(&req))
        },
        Err(_) => {
            Ok(HttpResponse::NotFound().json(StatusResponse {
                status: "error".to_string(),
                filename: None,
                error: Some("File not found".to_string()),
            }))
        }
    }
}

async fn save_file(mut field: Field, file_path: impl AsRef<Path>) -> std::io::Result<()> {
    let mut file = fs::File::create(file_path)?;
    
    while let Some(chunk_result) = field.next().await {
        match chunk_result {
            Ok(data) => file.write_all(&data)?,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        }
    }
    
    Ok(())
}
