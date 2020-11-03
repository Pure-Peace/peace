use actix_web::web::{Bytes, Data, Form, Query};
use actix_web::{get, post, HttpRequest, HttpResponse, Responder};
use prometheus::IntCounterVec;
use serde::Deserialize;

use actix_multipart::Multipart;
use std::borrow::BorrowMut;
use crate::utils;

#[derive(Debug, Deserialize)]
pub struct Lastfm {
    b: String,
    action: String,
    us: String,
    ha: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckUpdates {
    action: String,
    stream: String,
    time: String,
}

#[derive(Debug, Deserialize)]
pub struct BanchoConnect {
    v: String,
    u: String,
    h: String,
    fx: String,
    ch: String,
    retry: u32,
}

#[derive(Debug, Deserialize)]
pub struct OsuSession {
    u: String,
    h: String,
    action: String,
}

#[get("/lastfm.php")]
pub async fn lastfm(
    req: HttpRequest,
    Query(query): Query<Lastfm>,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    let success = || {
        counter
            .with_label_values(&["/lastfm.php", "get", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };

    println!("query: {:?}", query);
    // Not flag
    if &query.b[0..1] != "a" {
        return success();
    }

    success()
}
//{'b', 'action', 'us', 'ha'}

#[get("/check-updates.php")]
pub async fn check_updates(
    req: HttpRequest,
    Query(query): Query<CheckUpdates>,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    let success = || {
        counter
            .with_label_values(&["/check-updates.php", "get", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };
    println!("query: {:?}", query);

    success()
}

#[post("/osu-session.php")]
pub async fn osu_session(
    req: HttpRequest,
    mut form: Multipart,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    let success = || {
        counter
            .with_label_values(&["/osu-session.php", "post", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };
    let data: OsuSession = utils::get_form_data(form.borrow_mut()).await;
    println!("{:?}", data);

    success()
}

#[post("/osu-error.php")]
pub async fn osu_error(
    req: HttpRequest,
    body: Bytes,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    let success = || {
        counter
            .with_label_values(&["/osu-error.php", "post", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };
    //println!("body: {:?}", body);

    success()
}

#[get("/bancho_connect.php")]
pub async fn bancho_connect(
    req: HttpRequest,
    Query(query): Query<BanchoConnect>,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    let success = || {
        counter
            .with_label_values(&["/bancho_connect.php", "get", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };
    println!("query: {:?}", query);

    success()
}
