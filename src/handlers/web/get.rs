use actix_web::{
    web::{Data, Query},
    HttpRequest, HttpResponse,
};
use async_std::sync::RwLock;
use serde::Deserialize;

use crate::{
    constants::{GameMode, ScoreboardType},
    database::Database,
    objects::{PlayMods, PlayerSessions},
    routes::web::Context,
    types::Argon2Cache,
};

#[inline(always)]
pub async fn osu_osz2_get_scores<'a>(ctx: &Context<'a>) -> HttpResponse {
    #[derive(Debug, Deserialize)]
    struct GetScores {
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

    let failed = HttpResponse::Ok().body("-1|false");

    // Parse query
    let mut data = match Query::<GetScores>::from_query(ctx.req.query_string()) {
        Ok(Query(data)) => data,
        Err(_) => {
            return failed;
        }
    };

    // Update game mode with playmod list (rx / ap)
    data.game_mode.update_with_playmod(&data.play_mods.list);
    debug!("osu-osz2-getscores: {:?}", data);

    // Get login
    let player = match ctx
        .player_sessions
        .read()
        .await
        .get_login_by_name(&data.username, &data.password_hash, &ctx.argon2_cache)
        .await
    {
        Some(p) => p,
        None => {
            return failed;
        }
    };

    // Try update user stats
    let update_result = player
        .write()
        .await
        .update_mods(&data.game_mode, &data.play_mods)
        .await;
    if let Some(user_stats) = update_result {
        ctx.player_sessions
            .read()
            .await
            .enqueue_all(&user_stats)
            .await;
    }

    // TODO: pp scoreboard or not? get settings from player object

    // TODO: get beatmap by

    // unimplemented("osu-osz2-getscores.php")
    return failed;
}
