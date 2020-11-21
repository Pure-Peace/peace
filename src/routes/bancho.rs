#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Bytes, Data};
use actix_web::{HttpRequest, HttpResponse, Responder};
use prometheus::IntCounterVec;

use crate::{database::Database, handlers::bancho};
use crate::objects::{Player, PlayerSessions};
use crate::utils;

pub async fn get(req: HttpRequest, body: Bytes, counter: Data<IntCounterVec>) -> impl Responder {
    counter
        .with_label_values(&["/bancho", "get", "start"])
        .inc();
    //println!("GET Body {:?}", &body);
    //println!("REQ {:?}\n--------------", req);
    //let contents = "Hello bancho!";
    HttpResponse::Ok().body(body)
}

pub async fn post(
    req: HttpRequest,
    body: Bytes,
    player_sessions: Data<PlayerSessions>,
    database: Data<Database>,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    // Prom counter
    counter
        .with_label_values(&["/bancho", "post", "start"])
        .inc();

    // Get headers
    let headers = req.headers();

    // Get real request ip
    let request_ip = utils::get_realip(&req)
        .await
        .expect("Cannot get request ip");

    // Get osu ver
    let osu_version = utils::get_osuver(&req).await;

    // If not login
    if !headers.contains_key("osu-token") {
        let (resp_body, token) = bancho::login(req, &body, request_ip, osu_version, &database, player_sessions).await;
        return HttpResponse::Ok()
            .set_header("cho-token", token)
            .set_header("cho-protocol", "19")
            .body(resp_body);
    }

    HttpResponse::Ok().body(body)
}
