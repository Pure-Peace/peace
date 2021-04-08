use actix_multipart::Multipart;
use actix_web::HttpResponse;

use serde::Deserialize;
use std::time::Instant;

use crate::objects::{ScoreData, SubmitModular};
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
pub async fn osu_error<'a>(_ctx: &Context<'a>, _payload: Multipart) -> HttpResponse {
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
    let failed = HttpResponse::Ok().body("ok");
    let submit_ip = match utils::get_realip(&ctx.req).await {
        Ok(ip) => ip,
        Err(_) => {
            error!("[osu_submit_modular] Failed to get submit ip address, refused.");
            return failed;
        }
    };
    debug!(
        "[osu_submit_modular] ip {} wants submit scores..",
        submit_ip
    );
    // Check Token
    if !utils::osu_sumit_token_checker(&ctx.req) {
        // TODO: maybe ban someone.
        warn!(
            "[osu_submit_modular] Invalid submit token, ip: {}",
            submit_ip
        );
        return failed;
    };
    // Parse submit mutipart data
    let submit_parse = Instant::now();
    let submit_data = match SubmitModular::from_mutipart(utils::get_mutipart_data(payload).await) {
        Some(d) => d,
        None => {
            warn!(
                "[osu_submit_modular] Failed to parse mutipart data, ip: {}",
                submit_ip
            );
            return failed;
        }
    };
    debug!(
        "[osu_submit_modular] submit data parse done, time spent: {:?}",
        submit_parse.elapsed()
    );
    // Parse score data
    let score_parse = Instant::now();
    let score_data = match ScoreData::from_submit_modular(&&submit_data).await {
        Some(s) => s,
        None => {
            warn!(
                "[osu_submit_modular] Failed to parse score data, ip: {}",
                submit_ip
            );
            return failed;
        }
    };
    debug!(
        "[osu_submit_modular] score data parse done, time spent: {:?}",
        score_parse.elapsed()
    );
    println!("submit: {:?};\nscore: {:?}", submit_data, score_data);
    HttpResponse::Ok().body("666")
}
