use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, HttpResponse, Responder};
use prometheus::IntCounterVec;

use crate::utils;

use crate::handlers::bancho;

pub async fn get_main(
    req: HttpRequest,
    body: Bytes,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    counter
        .with_label_values(&["/bancho", "get", "start"])
        .inc();
    //println!("GET Body {:?}", &body);
    //println!("REQ {:?}\n--------------", req);
    //let contents = "Hello bancho!";
    HttpResponse::Ok().body(body)
}

pub async fn post_main(
    req: HttpRequest,
    body: Bytes,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    // Prom counter
    counter
        .with_label_values(&["/bancho", "post", "start"])
        .inc();

    // Get headers
    let headers = req.headers();

    // Get real request ip
    let request_ip = utils::get_realip(&req).await.expect("Cannot get request ip");

    // Get osu ver
    let osu_version = utils::get_osuver(&req).await;

    // If not login
    if !headers.contains_key("osu-token") {
        let (resp_body, token) = bancho::login(&body, request_ip, osu_version).await;
        return HttpResponse::Ok()
            .set_header("cho-token", token)
            .set_header("cho-protocol", "19")
            .body(resp_body);
    }

    HttpResponse::Ok().body(body)
}
