#![allow(unused_imports)]
#![allow(unused_variables)]
use actix_multipart::Multipart;
use actix_web::HttpResponse;
use serde::Deserialize;

use crate::routes::web::Context;
use crate::utils;

#[inline(always)]
/// Multipart Form-data
///
/// POST /web/osu-error.php
///
/// ```
/// OsuError {
///     u: String = username,
///     p: String = password hash,
///     i: i32,
///     osumode: String = [Menu],
///     gamemode: String = [Osu, Taiko, Mania, Catch],
///     gametime: u32,
///     audiotime: u32,
///     culture: String = [zh-CN],
///     beatmap_id: u32,
///     beatmap_checksum: String,
///     exception: String = [System.Exception],
///     feedback: String = [update error],
///     stacktrace: String,
///     soft: String = [True, False],
///     beatmap_count: u32,
///     compatibility: u32,
///     version: String = osu version,
///     exehash: String,
///     config: String = osu config(ini),
/// }
///
/// ```
pub async fn osu_error<'a>(ctx: &Context<'a>, payload: Multipart) -> HttpResponse {
    #[derive(Debug, Deserialize)]
    struct OsuError {
        u: String,
        p: String,
        i: i32,
        osumode: String,
        gamemode: String,
        gametime: u32,
        audiotime: u32,
        culture: String,
        beatmap_id: u32,
        beatmap_checksum: String,
        exception: String,
        feedback: String,
        stacktrace: String,
        soft: String,
        beatmap_count: u32,
        compatibility: u32,
        version: String,
        exehash: String,
        config: String,
    }

    // parse osu errors
    // currently unused
    /* let data = utils::get_form_data::<OsuError>(payload).await;
    error!("{:?}", data); */

    HttpResponse::Ok().body("ok")
}

#[inline(always)]
pub async fn osu_submit_modular<'a>(ctx: &Context<'a>, payload: Multipart) -> HttpResponse {
    HttpResponse::Ok().body("666")
}
