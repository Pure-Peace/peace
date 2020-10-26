use actix_web::{get, HttpResponse, Responder};

/// GET "/bancho"
#[get("")]
pub async fn main() -> impl Responder {
    let contents = "Hello bancho!";
    HttpResponse::Ok().body(contents)
}
