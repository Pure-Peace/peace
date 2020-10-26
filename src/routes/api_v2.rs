use actix_web::{get, HttpResponse, Responder};

/// GET "/api/v2"
#[get("")]
pub async fn index() -> impl Responder {
    let contents = "Hello api v2!";
    HttpResponse::Ok().body(contents)
}
