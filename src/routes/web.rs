use actix_web::web::{Bytes, Data, Form, Query};
use actix_web::{get, post, HttpRequest, HttpResponse, Responder};
use prometheus::IntCounterVec;
use serde::Deserialize;

use crate::utils;
use actix_multipart::Multipart;
use std::borrow::BorrowMut;

/// Query Data
///
/// GET /web/lastfm.php
///
/// ```
/// Lastfm {
///     b: String = beatmap ban,
///     action: String,
///     us: String = username,
///     ha: String = password hash,
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct Lastfm {
    b: String,
    action: String,
    us: String,
    ha: String,
}

/// Query Data
///
/// GET /web/check-updates.php
///
/// ```
/// CheckUpdates {
///     action: String = [check, path, error],
///     stream: String = [cuttingedge, stable40, beta40, stable],
///     time: String = timeStamp,
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct CheckUpdates {
    action: String,
    stream: String,
    time: String,
}

/// Query Data
///
/// GET /web/bancho_connect.php
///
/// ```
/// BanchoConnect {
///     v: String = osu version,
///     u: String = username,
///     h: String = password hash,
///     fx: String = donet env,
///     ch: String = hardware hashes,
///     retry: i32 = retries,
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct BanchoConnect {
    v: String,
    u: String,
    h: String,
    fx: String,
    ch: String,
    retry: i32,
}

/// Multipart Form-data
///
/// POST /web/osu-session.php
///
/// ```
/// OsuSession {
///     u: String = username,
///     h: String = password hash,
///     action: String = [check, submit],
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct OsuSession {
    u: String,
    h: String,
    action: String,
}

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
#[derive(Debug, Deserialize)]
pub struct OsuError {
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
        HttpResponse::Ok().body(Bytes::from(r#"[{"file_version":"3461","filename":"avcodec-51.dll","file_hash":"b66478cc0f9ec50810489a039ced642b","filesize":"4426976","timestamp":"2020-06-26 05:54:54","patch_id":null,"url_full":"http:\/\/m3.ppy.sh\/r\/avcodec-51.dll\/f_b66478cc0f9ec50810489a039ced642b"},{"file_version":"3462","filename":"avformat-52.dll","file_hash":"c00b30289cc427caff97af5aa3d43e03","filesize":"728800","timestamp":"2020-06-26 05:56:16","patch_id":null,"url_full":"http:\/\/m1.ppy.sh\/r\/avformat-52.dll\/f_c00b30289cc427caff97af5aa3d43e03"},{"file_version":"3511","filename":"avutil-49.dll","file_hash":"47c83b958951331ba409d6b80316250c","filesize":"79616","timestamp":"2020-08-05 03:30:35","patch_id":null,"url_full":"http:\/\/m3.ppy.sh\/r\/avutil-49.dll\/f_47c83b958951331ba409d6b80316250c"},{"file_version":"3379","filename":"bass.dll","file_hash":"7623474a8b9bec1e3ffca813cdf93bc3","filesize":"128181","timestamp":"2020-02-07 02:05:33","patch_id":null,"url_full":"http:\/\/m2.ppy.sh\/r\/bass.dll\/f_7623474a8b9bec1e3ffca813cdf93bc3"},{"file_version":"3464","filename":"bass_fx.dll","file_hash":"3ad3c0fd4dca001a2f9e707b74544919","filesize":"51512","timestamp":"2020-06-26 05:57:49","patch_id":null,"url_full":"http:\/\/m1.ppy.sh\/r\/bass_fx.dll\/f_3ad3c0fd4dca001a2f9e707b74544919"},{"file_version":"3591","filename":"Microsoft.Ink.dll","file_hash":"82d4ee89f4a39c764fa6297a95ebb10e","filesize":"467712","timestamp":"2020-10-17 03:51:17","patch_id":null,"url_full":"http:\/\/m3.ppy.sh\/r\/Microsoft.Ink.dll\/f_82d4ee89f4a39c764fa6297a95ebb10e"},{"file_version":"3605","filename":"osu!.exe","file_hash":"a0a4ec1067d63cc1c619a99543120af6","filesize":"4396800","timestamp":"2020-11-02 12:36:58","patch_id":null,"url_full":"http:\/\/m2.ppy.sh\/r\/osu!.exe\/f_a0a4ec1067d63cc1c619a99543120af6"},{"file_version":"3453","filename":"osu!ui.dll","file_hash":"5faec5d47fedf5bc82afac282ba990b5","filesize":"25776864","timestamp":"2020-06-25 02:14:18","patch_id":null,"url_full":"http:\/\/m3.ppy.sh\/r\/osu!ui.dll\/f_5faec5d47fedf5bc82afac282ba990b5"},{"file_version":"3470","filename":"pthreadGC2.dll","file_hash":"00678eb6be3b52d562b66218c93e21a8","filesize":"77400","timestamp":"2020-06-26 06:06:51","patch_id":null,"url_full":"http:\/\/m2.ppy.sh\/r\/pthreadGC2.dll\/f_00678eb6be3b52d562b66218c93e21a8"},{"file_version":"3460","filename":"osu!gameplay.dll","file_hash":"4cb98d63f1b2b9dc38e10e9901ec52d8","filesize":"31854304","timestamp":"2020-06-26 05:49:08","patch_id":null,"url_full":"http:\/\/m1.ppy.sh\/r\/osu!gameplay.dll\/f_4cb98d63f1b2b9dc38e10e9901ec52d8"},{"file_version":"3468","filename":"OpenTK.dll","file_hash":"b4d949571134fc3ec6c28f1af7a75e49","filesize":"4368096","timestamp":"2020-06-26 06:03:38","patch_id":null,"url_full":"http:\/\/m1.ppy.sh\/r\/OpenTK.dll\/f_b4d949571134fc3ec6c28f1af7a75e49"},{"file_version":"1753","filename":"d3dcompiler_47.dll","file_hash":"c5b362bce86bb0ad3149c4540201331d","filesize":"3466856","timestamp":"2015-08-14 08:35:25","patch_id":null,"url_full":"http:\/\/m2.ppy.sh\/r\/d3dcompiler_47.dll\/f_c5b362bce86bb0ad3149c4540201331d"},{"file_version":"3465","filename":"libEGL.dll","file_hash":"9f7f22cef980ec272a9b73bf317500e4","filesize":"150240","timestamp":"2020-06-26 05:58:53","patch_id":null,"url_full":"http:\/\/m3.ppy.sh\/r\/libEGL.dll\/f_9f7f22cef980ec272a9b73bf317500e4"},{"file_version":"3466","filename":"libGLESv2.dll","file_hash":"a4dfddff62d1e917ebb0688cf8d96be7","filesize":"3368160","timestamp":"2020-06-26 05:59:23","patch_id":null,"url_full":"http:\/\/m3.ppy.sh\/r\/libGLESv2.dll\/f_a4dfddff62d1e917ebb0688cf8d96be7"},{"file_version":"3571","filename":"osu!seasonal.dll","file_hash":"184896bee1c06f0b9616e4c5ceb8683e","filesize":"5219584","timestamp":"2020-10-11 17:04:58","patch_id":null,"url_full":"http:\/\/m1.ppy.sh\/r\/osu!seasonal.dll\/f_184896bee1c06f0b9616e4c5ceb8683e"},{"file_version":"3596","filename":"osu!auth.dll","file_hash":"a3f75a5050e20f890bede92c3e684f15","filesize":"9563904","timestamp":"2020-10-18 15:52:03","patch_id":"3597","url_full":"http:\/\/m1.ppy.sh\/r\/osu!auth.dll\/f_a3f75a5050e20f890bede92c3e684f15","url_patch":"http:\/\/m1.ppy.sh\/r\/osu!auth.dll\/p_a3f75a5050e20f890bede92c3e684f15_b38fc47251009eb9b462e63d8264176b"}]"#))
    };
    println!("query: {:?}", query);

    success()
}

#[post("/osu-session.php")]
pub async fn osu_session(
    req: HttpRequest,
    mut formData: Multipart,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    let success = || {
        counter
            .with_label_values(&["/osu-session.php", "post", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from(""))
    };
    let data: OsuSession = utils::get_form_data(formData.borrow_mut()).await;
    println!("{:?}", data);

    success()
}

#[post("/osu-error.php")]
pub async fn osu_error(
    req: HttpRequest,
    mut formData: Multipart,
    counter: Data<IntCounterVec>,
) -> impl Responder {
    let success = || {
        counter
            .with_label_values(&["/osu-error.php", "post", "success"])
            .inc();
        HttpResponse::Ok().body(Bytes::from("-3"))
    };
    let data: OsuError = utils::get_form_data(formData.borrow_mut()).await;
    //println!("{:?}", data);

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
        HttpResponse::Ok().body(Bytes::from("error: pass"))
    };
    println!("query: {:?}", query);

    success()
}
