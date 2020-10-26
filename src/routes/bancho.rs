use actix_web::{get, HttpResponse, Responder};

/// GET "/"
#[get("")]
pub async fn main() -> impl Responder {
    let contents = "Hello bancho!";
    HttpResponse::Ok().body(contents)
}
