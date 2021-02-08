#![allow(unused_variables)]

use actix_web::web::{Bytes, Data, Query};
use actix_web::{get, post, HttpRequest, HttpResponse};
use prometheus::IntCounterVec;

use crate::utils;
use actix_multipart::Multipart;

use super::data::*;

#[get("/lastfm.php")]
pub async fn lastfm(
    req: HttpRequest,
    Query(query): Query<Lastfm>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    let success = || {
        counter
            .with_label_values(&["/lastfm.php", "get", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };

    debug!("query: {:?}", query);
    // Not flag
    if &query.b[0..1] != "a" {
        return success();
    }

    success()
}

#[get("/check-updates.php")]
pub async fn check_updates(
    req: HttpRequest,
    Query(query): Query<CheckUpdates>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    let success = || {
        counter
            .with_label_values(&["/check-updates.php", "get", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from(r#"[]"#))
    };
    debug!("query: {:?}", query);

    success()
}

#[post("/osu-session.php")]
pub async fn osu_session(
    req: HttpRequest,
    form_data: Multipart,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    let success = || {
        counter
            .with_label_values(&["/osu-session.php", "post", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from(""))
    };
    let data: OsuSession = match utils::get_form_data(form_data).await {
        Ok(data) => data,
        Err(_) => {
            return HttpResponse::Ok().body(Bytes::from(""));
        }
    };
    debug!("{:?}", data);

    success()
}

#[post("/osu-error.php")]
pub async fn osu_error(
    req: HttpRequest,
    form_data: Multipart,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    let success = || {
        counter
            .with_label_values(&["/osu-error.php", "post", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };
    let data: OsuError = match utils::get_form_data(form_data).await {
        Ok(data) => data,
        Err(_) => {
            return HttpResponse::Ok().body(Bytes::from(""));
        }
    };
    //println!("{:?}", data);

    success()
}

#[get("/bancho_connect.php")]
pub async fn bancho_connect(
    req: HttpRequest,
    Query(query): Query<BanchoConnect>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    let success = || {
        counter
            .with_label_values(&["/bancho_connect.php", "get", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("error: pass"))
    };
    debug!("query: {:?}", query);

    success()
}
