use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn index() -> impl Responder {
    let contents = "Hello!";
    HttpResponse::Ok().body(contents)
}
