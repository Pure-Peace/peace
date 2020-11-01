use actix_web::web::{Bytes, Data};
use actix_web::{get, post, HttpResponse, Responder};
use prometheus::IntCounterVec;

/// GET
#[get("*")]
pub async fn get_main(counter: Data<IntCounterVec>, body: Bytes) -> impl Responder {
    counter.with_label_values(&["/bancho", "get", "start"]).inc();
    println!("GET Body {:?}!", &body);
    //let contents = "Hello bancho!";
    HttpResponse::Ok().body(body)
}

/// POST
#[post("*")]
pub async fn post_main(counter: Data<IntCounterVec>, body: Bytes) -> impl Responder {
    counter.with_label_values(&["/bancho", "post", "start"]).inc();
    println!("POST Body {:?}!", &body);
    //let contents = "Hello bancho!";
    HttpResponse::Ok().body(body)
}
