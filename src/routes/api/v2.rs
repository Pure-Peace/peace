use actix_web::{web::Data, HttpResponse, Responder, get};
use prometheus::IntCounterVec;

/// GET "/api/v2"
#[get("")]
pub async fn index(counter: Data<IntCounterVec>) -> impl Responder {
    counter.with_label_values(&["eee", "555", "444"]).inc();
    let contents = "Hello api v2!";
    HttpResponse::Ok().body(contents)
}
