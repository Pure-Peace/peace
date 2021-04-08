#![allow(unused_imports)]
#![allow(unused_variables)]
use actix_multipart::Multipart;
use actix_web::HttpMessage;
use actix_web::{HttpRequest, HttpResponse};
use base64::decode;
use bytes::Bytes;
use pyo3::{
    types::{PyBytes, PyDict},
    PyErr, Python,
};
use serde::Deserialize;
use std::time::Instant;

use crate::utils::{self, MultipartData};
use crate::{constants::GameMode, objects::PlayMods, routes::web::Context};

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

    #[inline(always)]
    /// Because Rust does not have an implementation of the rijndael algorithm,
    /// it is temporarily solved with the built-in python3 interpreter.
    pub fn python_decrypt(&self) -> Result<Vec<String>, PyErr> {
        debug!("[SubmitModular] Python decrypt start");
        let start = Instant::now();
        let gil = Python::acquire_gil();
        let python = gil.python();
        let module = python.import("__main__")?;

        let decryp_result = module
            .call_method1(
                "rijndael_cbc_decrypt",
                (
                    format!("osu!-scoreburgr---------{}", self.osu_version),
                    PyBytes::new(python, &self.iv),
                    PyBytes::new(python, &self.score),
                ),
            )?
            .extract()?;
        let end = start.elapsed();
        debug!(
            "[SubmitModular] Python decrypt success, time spent: {:?}",
            end
        );
        return Ok(decryp_result);
    }
}

#[derive(Debug)]
pub struct ScoreData {
    beatmap_md5: String,
    player_name: String,
    score_md5: String,
    n300: i32,
    n100: i32,
    n50: i32,
    geki: i32,
    katu: i32,
    miss: i32,
    score: i32,
    max_combo: i32,
    perfect: bool,
    grade: String,
    mods: PlayMods,
    pass: bool,
    mode: GameMode,
    osu_version: i32,
    client_flags: i32,
}

impl ScoreData {
    #[inline(always)]
    pub async fn from_submit_modular(submit_data: &SubmitModular) -> Option<Self> {
        use utils::try_parse;
        let data = match submit_data.python_decrypt() {
            Ok(d) => d,
            Err(err) => {
                warn!("[SubmitModular] Python decrypt failed, err: {:?}", err);
                return None;
            }
        };
        // Check len
        if data.len() < 18 {
            warn!("[SubmitModular] Invalid score data length ( < 18)");
            return None;
        };
        let player_name = data[1].trim().to_string();
        // Check beatmap md5
        let beatmap_md5 = data[0].to_string();
        if beatmap_md5.len() != 32 || beatmap_md5 != submit_data.beatmap_hash {
            warn!(
                "[SubmitModular] Refused: {}; decrypted submit beatmap hash({}) not equal({}).",
                player_name, beatmap_md5, submit_data.beatmap_hash
            );
            return None;
        };
        // Check osu version
        let osu_version = try_parse::<i32>(&data[17][..8])?;
        if osu_version != submit_data.osu_version {
            warn!(
                "[SubmitModular] Refused: {}; decrypted osu version({}) not equal({}).",
                player_name, osu_version, submit_data.osu_version
            );
            return None;
        }
        let client_flags = {
            let mut count = 0;
            for i in data[17].chars() {
                if i == ' ' {
                    count += 1;
                }
            }
            count
        };
        let mods = PlayMods::parse(try_parse(&data[13])?);
        let mode = GameMode::parse_with_playmod(try_parse(&data[15])?, &mods.list)?;

        Some(Self {
            beatmap_md5: data[0].to_string(),
            player_name,
            score_md5: data[2].to_string(),
            n300: try_parse(&data[3])?,
            n100: try_parse(&data[4])?,
            n50: try_parse(&data[5])?,
            geki: try_parse(&data[6])?,
            katu: try_parse(&data[7])?,
            miss: try_parse(&data[8])?,
            score: try_parse(&data[9])?,
            max_combo: try_parse(&data[10])?,
            perfect: &data[11] == "True",
            grade: data[12].to_string(),
            mods,
            pass: &data[14] == "True",
            mode,
            osu_version,
            client_flags,
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
