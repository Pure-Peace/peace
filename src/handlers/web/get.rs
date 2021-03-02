use actix_web::{web::Query, HttpResponse};
use serde::Deserialize;

use crate::{
    constants::{GameMode, ScoreboardType},
    objects::PlayMods,
    routes::web::Context,
};

macro_rules! get_login {
    ($ctx:ident, $data:ident, $failed:ident) => {
        match $ctx
            .player_sessions
            .read()
            .await
            .get_login_by_name(&$data.username, &$data.password_hash, &$ctx.argon2_cache)
            .await
        {
            Some(p) => p,
            None => {
                return $failed;
            }
        }
    };
}

macro_rules! parse_query {
    ($ctx:ident, $typ:ty, $failed:ident) => {
        match Query::<$typ>::from_query($ctx.req.query_string()) {
            Ok(Query(data)) => data,
            Err(_) => {
                return $failed;
            }
        }
    };
}


#[inline(always)]
pub async fn osu_get_friends<'a>(ctx: &Context<'a>) -> HttpResponse {
    let failed = HttpResponse::Unauthorized().body("");
    #[derive(Debug, Deserialize)]
    struct GetFriends {
        #[serde(rename = "u")]
        username: String,
        #[serde(rename = "h")]
        password_hash: String,
    }

    // Parse query
    let data = parse_query!(ctx, GetFriends, failed);
    // Get login
    let player = get_login!(ctx, data, failed);

    // Convert to string
    let friends: String = player
        .read()
        .await
        .friends
        .iter()
        .map(|id| id.to_string() + "\n")
        .collect();

    HttpResponse::Ok().body(friends)
}

#[inline(always)]
/// Seasonal background images
///
/// Locate on database -> bancho.config.seasonal_backgrounds
///
/// String Array
pub async fn osu_get_seasonal<'a>(ctx: &Context<'a>) -> HttpResponse {
    if let Some(background_images) = &ctx.bancho_config.read().await.seasonal_backgrounds {
        return HttpResponse::Ok().json(background_images);
    };

    HttpResponse::Ok().body("[]")
}

#[inline(always)]
pub async fn osu_osz2_get_scores<'a>(ctx: &Context<'a>) -> HttpResponse {
    let failed = HttpResponse::Ok().body("-1|false");
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

    // Parse query
    let mut data = parse_query!(ctx, GetScores, failed);
    // Update game mode with playmod list (rx / ap)
    data.game_mode.update_with_playmod(&data.play_mods.list);
    // Get login
    let player = get_login!(ctx, data, failed);

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
