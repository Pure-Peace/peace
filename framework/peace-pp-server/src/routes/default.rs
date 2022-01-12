use askama::Template;
use ntex::web::{get, types::Data, HttpResponse};

use crate::objects::glob::Glob;

/// GET "/"
#[get("/")]
pub async fn index(glob: Data<Glob>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(glob.render_main_page.render().unwrap())
}
