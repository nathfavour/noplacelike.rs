use actix_web::{get, web, HttpResponse, Scope};

use crate::templates;

// Create UI scope
pub fn ui_scope() -> Scope {
    web::scope("/ui")
        .service(home)
}

#[get("/")]
async fn home() -> HttpResponse {
    let template = templates::HomeTemplate {};
    templates::render_template(&template)
}

pub async fn redirect_to_ui() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/ui"))
        .finish()
}
