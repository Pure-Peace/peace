use actix_web::web::{Bytes, Data};
use actix_web::{get, post, HttpRequest, HttpResponse, Responder};
use prometheus::IntCounterVec;

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
    println!("REQ {:?}\n--------------", req);
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

    // Print
    //println!("POST Body {:?}", &body);
    //println!("REQ {:?}\n--------------", req);
    // Get headers
    let headers = req.headers();

    // Get real request ip
    let request_ip = match req.connection_info().realip_remote_addr() {
        Some(ip) => ip.to_string(),
        None => return HttpResponse::NonAuthoritativeInformation().body("wtf"),
    };

    // Get osu ver
    let osu_version = match headers.get("osu-version") {
        Some(version) => version.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    };

    // If not login
    if !headers.contains_key("osu-token") {
        let (resp_body, token) = bancho::login(&body, request_ip, osu_version).await;
        return HttpResponse::Ok()
            .set_header("cho-token", token)
            .body(resp_body);
    }

    HttpResponse::Ok().body(body)
}
