use actix_web::HttpResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {}

pub fn render_template<T>(template: &T) -> HttpResponse
where
    T: Template,
{
    match template.render() {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => {
            eprintln!("Template error: {}", e);
            HttpResponse::InternalServerError().body("Template error")
        }
    }
}
