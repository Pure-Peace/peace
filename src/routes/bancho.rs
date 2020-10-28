use actix_web::{get, HttpResponse, Responder};
use actix_web::web::Data;
use prometheus::IntCounterVec;

/// GET "/bancho"
#[get("")]
pub async fn main(counter: Data<IntCounterVec>) -> impl Responder {
    counter.with_label_values(&["orz", "gg", "666"]).inc();
    let contents = "Hello bancho!";
    HttpResponse::Ok().body(contents)
}
