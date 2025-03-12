use actix_web::{delete, get, post, web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};

use crate::config::{add_audio_folder, load_config, remove_audio_folder};
use crate::templates;

#[derive(Debug, Serialize)]
struct DirsResponse {
    dirs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DirRequest {
    dir: String,
}

// Create admin scope
pub fn admin_scope() -> Scope {
    web::scope("/admin")
        .service(admin_panel)
        .service(get_dirs)
        .service(add_dir)
        .service(remove_dir)
}

#[get("/")]
async fn admin_panel() -> HttpResponse {
    let template = templates::AdminTemplate {};
    templates::render_template(&template)
}

#[get("/dirs")]
async fn get_dirs() -> HttpResponse {
    let config = load_config();
    
    HttpResponse::Ok().json(DirsResponse {
        dirs: config.audio_folders,
    })
}

#[post("/dirs")]
async fn add_dir(req: web::Json<DirRequest>) -> HttpResponse {
    match add_audio_folder(req.dir.clone()) {
        Ok(_) => HttpResponse::Ok().json(StatusResponse {
            status: "success".to_string(),
            error: None,
        }),
        Err(e) => HttpResponse::BadRequest().json(StatusResponse {
            status: "error".to_string(),
            error: Some(e),
        }),
    }
}

#[delete("/dirs")]
async fn remove_dir(req: web::Json<DirRequest>) -> HttpResponse {
    match remove_audio_folder(&req.dir) {
        Ok(_) => HttpResponse::Ok().json(StatusResponse {
            status: "success".to_string(),
            error: None,
        }),
        Err(e) => HttpResponse::BadRequest().json(StatusResponse {
            status: "error".to_string(),
            error: Some(e),
        }),
    }
}
