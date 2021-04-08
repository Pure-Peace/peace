#![allow(unused_imports)]
#![allow(unused_variables)]
use actix_multipart::Multipart;
use actix_web::HttpMessage;
use actix_web::{HttpRequest, HttpResponse};
use bytes::Bytes;
use serde::Deserialize;

use crate::routes::web::Context;
use crate::utils::{self, MultipartData};

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

#[derive(Debug)]
pub struct SubmitModular {
    pub quit: bool,     // x (quit 0 or 1)
    pub fail_time: i32, // ft (fail time)
    pub score: Vec<u8>, // score (base64 -> bytes)
    pub fs: String,
    pub beatmap_hash: String, // bmk
    pub c1: String,
    pub st: i32,
    pub password: String,    // pass (password)
    pub osu_version: i32,    // osuver
    pub client_hash: String, // s (client_hash)
    pub iv: Vec<u8>,         // iv (initialization vector base64 - bytes)

    pub score_data: Bytes, // score (replay file, octet-stream bytes)
}

impl SubmitModular {
    #[inline(always)]
    pub fn from_mutipart(mut data: MultipartData) -> Option<Self> {
        Some(Self {
            quit: data.form::<i32>("x")? == 1,
            fail_time: data.form("ft")?,
            score: match decode(data.form::<String>("score")?) {
                Ok(s) => s,
                Err(e) => return None,
            },
            fs: data.form("fs")?,
            beatmap_hash: data.form("bmk")?,
            c1: data.form("c1")?,
            st: data.form("st")?,
            password: data.form("pass")?,
            osu_version: data.form("osuver")?,
            client_hash: data.form("s")?,
            iv: match decode(data.form::<String>("iv")?) {
                Ok(s) => s,
                Err(e) => return None,
            },
            score_data: data.file("score")?,
        })
    }

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

    if !utils::osu_sumit_token_checker(&ctx.req) {
        // TODO: maybe ban someone.
        warn!(
            "[osu_submit_modular] Invalid submit token, ip: {}",
            submit_ip
        );
        return failed;
    };
    let data = match SubmitModular::from_mutipart(utils::get_mutipart_data(payload).await) {
        Some(d) => d,
        None => {
            warn!(
                "[osu_submit_modular] Failed to parse submit data, ip: {}",
                submit_ip
            );
            return failed;
        }
    };
    println!("{:?}", data);
    HttpResponse::Ok().body("666")
}
