use actix_web::{get, HttpResponse, Responder};

/// GET "/"
#[get("/")]
pub async fn index() -> impl Responder {
    let contents = "Hello root!";
    HttpResponse::Ok().body(contents)
}
