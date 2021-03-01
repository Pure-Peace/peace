#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Data, Query};
use actix_web::{get, HttpRequest, HttpResponse};
use async_std::sync::RwLock;
use num_traits::FromPrimitive;
use prometheus::IntCounterVec;
use serde::Deserialize;

use crate::packets;
use crate::{constants::GameMode, database::Database};
use crate::{constants::ScoreboardType, objects::PlayerSessions};
use crate::{objects::PlayMods, types::Argon2Cache};
use crate::{settings::bancho::BanchoConfig, utils};

use super::data::*;

#[inline(always)]
fn unimplemented(route: &str) -> HttpResponse {
    warn!("[GET] Unimplemented route request: {}", route);
    HttpResponse::Ok().body("ok")
}

#[get("/check-updates.php")]
pub async fn check_updates(
    req: HttpRequest,
    Query(query): Query<CheckUpdates>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    unimplemented("check-updates.php")
}

#[get("/bancho_connect.php")]
pub async fn bancho_connect(req: HttpRequest, Query(query): Query<BanchoConnect>) -> HttpResponse {
    unimplemented("bancho_connect.php")
}

#[get("/lastfm.php")]
pub async fn lastfm(
    req: HttpRequest,
    Query(query): Query<Lastfm>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    unimplemented("lastfm.php")
}

#[get("/osu-rate.php")]
pub async fn osu_rate(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-rate.php")
}

#[get("/osu-addfavourite.php")]
pub async fn osu_add_favourite(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-addfavourite.php")
}

#[get("/osu-markasread.php")]
pub async fn osu_mark_as_read(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-markasread.php")
}

#[get("/osu-getreplay.php")]
pub async fn osu_get_replay(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-getreplay.php")
}
#[get("/osu-getfavourites.php")]
pub async fn osu_get_favourites(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-getfavourites.php")
}

#[get("/osu-getfriends.php")]
pub async fn osu_get_friends(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-getfriends.php")
}

#[get("/osu-getseasonal.php")]
/// Seasonal background images
///
/// Locate on database -> bancho.config.seasonal_backgrounds
/// String Array
pub async fn osu_get_seasonal(bancho_config: Data<RwLock<BanchoConfig>>) -> HttpResponse {
    if let Some(background_images) = &bancho_config.read().await.seasonal_backgrounds {
        return HttpResponse::Ok().json(background_images);
    };

    HttpResponse::Ok().body("[]")
}

#[get("osu-get-beatmap-topic.php")]
pub async fn osu_get_beatmap_topic(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-get-beatmap-topic.php")
}

#[get("/osu-search.php")]
pub async fn osu_search(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-search.php")
}

#[get("/osu-search-set.php")]
pub async fn osu_search_set(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-search-set.php")
}

#[derive(Debug, Deserialize)]
pub struct GetScores {
    // -
    s: i32,
    #[serde(rename = "vv")]
    scoreboard_version: i32,
    #[serde(rename = "v")]
    scoreboard_type: ScoreboardType,
    #[serde(rename = "c")]
    beatmap_hash: String,
    #[serde(rename = "f")]
    beatmap_filename: String,
    #[serde(rename = "m")]
    game_mode: GameMode,
    #[serde(rename = "i")]
    beatmap_set_id: i32,
    #[serde(rename = "mods")]
    play_mods: PlayMods,
    // -
    h: String,
    // -
    a: i32,
    #[serde(rename = "us")]
    username: String,
    #[serde(rename = "ha")]
    password_hash: String,
}

#[get("/osu-osz2-getscores.php")]
pub async fn osu_osz2_get_scores(
    req: HttpRequest,
    Query(mut data): Query<GetScores>,
    player_sessions: Data<RwLock<PlayerSessions>>,
    database: Data<Database>,
    argon2_cache: Data<RwLock<Argon2Cache>>,
) -> HttpResponse {
    let failed = HttpResponse::Ok().body("-1|false");
    debug!("osu-osz2-getscores: {:?}", data);

    // Update game mode with playmod list (rx / ap)
    data.game_mode.update_with_playmod(&data.play_mods.list);

    // Get login
    let player = match player_sessions
        .read()
        .await
        .get_login_by_name(&data.username, &data.password_hash, &argon2_cache)
        .await
    {
        Some(p) => p,
        None => {
            return failed;
        }
    };

    // Try update user stats
    if let Some(user_stats) = player
        .write()
        .await
        .update_mods(&data.game_mode, &data.play_mods)
        .await
    {
        player_sessions.read().await.enqueue_all(&user_stats).await;
    }

    // TODO: pp scoreboard or not? get settings from player object

    // TODO: get beatmap by

    // unimplemented("osu-osz2-getscores.php")
    return failed;
}

#[get("osu-osz2-bmsubmit-getid")]
pub async fn osu_osz2_bm_submit_getid(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-osz2-bmsubmit-getid")
}
