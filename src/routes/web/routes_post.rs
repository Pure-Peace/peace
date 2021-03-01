#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_multipart::Multipart;
use actix_web::{post, web::Query, HttpRequest, HttpResponse};
use prometheus::IntCounterVec;
use serde::Deserialize;

use crate::utils;

use super::data::*;

#[inline(always)]
fn unimplemented(route: &str) -> HttpResponse {
    warn!("[POST] Unimplemented route request: {}", route);
    HttpResponse::Ok().body("ok")
}

#[post("/osu-session.php")]
pub async fn osu_session(req: HttpRequest, form_data: Multipart) -> HttpResponse {
    unimplemented("osu-session.php")
}

#[post("/osu-error.php")]
pub async fn osu_error(req: HttpRequest, form_data: Multipart) -> HttpResponse {
    unimplemented("osu-error.php")
}

#[post("/osu-comment.php")]
pub async fn osu_comment(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-comment.php")
}
#[post("osu-osz2-bmsubmit-post.php")]
pub async fn osu_osz2_bm_submit_post(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-osz2-bmsubmit-post.php")
}

#[post("osu-osz2-bmsubmit-upload.php")]
pub async fn osu_osz2_bm_submit_upload(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-osz2-bmsubmit-upload.php")
}
#[post("/osu-screenshot.php")]
pub async fn osu_screenshot(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-screenshot.php")
}

#[post("/osu-getbeatmapinfo.php")]
pub async fn osu_get_beatmap_info(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-getbeatmapinfo.php")
}

#[post("/osu-submit-modular-selector.php")]
pub async fn osu_submit_modular(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-submit-modular-selector.php")
}
