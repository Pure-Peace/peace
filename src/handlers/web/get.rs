use actix_web::{web::Query, HttpResponse};
use async_std::fs::File;
use async_std::prelude::*;
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

const HACK_DETECTED_NOTIFICATION: Option<&str> = Some(
    "Warning: Please do not cheat. It makes no sense. If you do, you will most likely be banned.",
);

#[inline(always)]
pub async fn lastfm<'a>(ctx: &Context<'a>) -> HttpResponse {
    let done = HttpResponse::Ok().body("-3");
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
        #[serde(rename = "b")]
        beatmap_ban: String,
        action: String,
        #[serde(rename = "us")]
        username: String,
        #[serde(rename = "ha")]
        password_hash: String,
    }

    let data = parse_query!(ctx, Lastfm, done);
    warn!("lastfm: {:?}", data);

    // Have not client flags
    if !data.beatmap_ban.starts_with('a') {
        return done;
    }

    // Try get value
    let beatmap_ban = data.beatmap_ban.get(1..);
    if beatmap_ban.is_none() {
        return done;
    }

    // Try parse value
    let beatmap_ban = beatmap_ban.unwrap().parse::<i32>();
    if beatmap_ban.is_err() {
        return done;
    }
    let beatmap_ban = beatmap_ban.unwrap();

    let player = get_login!(ctx, data, done);
    let mut player = player.write().await;
    warn!(
        "lastfm detected hack {} by player {}({})",
        beatmap_ban, player.name, player.id
    );

    // Detected
    player
        .hack_detected(
            beatmap_ban,
            "lastfm",
            HACK_DETECTED_NOTIFICATION,
            &ctx.database,
        )
        .await;
    // TODO: may sent to discord hooks

    done
}

#[inline(always)]
pub async fn osu_get_replay<'a>(ctx: &Context<'a>) -> HttpResponse {
    const REPLAY_PATH: &'static str = ".data/replays";
    let failed = HttpResponse::Unauthorized().body("");
    #[derive(Debug, Deserialize)]
    struct GetReplay {
        #[serde(rename = "u")]
        username: String,
        #[serde(rename = "h")]
        password_hash: String,
        #[serde(rename = "c")]
        score_id: u64,
    }

    // Parse query
    let data = parse_query!(ctx, GetReplay, failed);
    // Get login
    let _player = get_login!(ctx, data, failed);

    if let Ok(mut file) = File::open(format!("{}/{}.osr", REPLAY_PATH, data.score_id)).await {
        let mut contents = Vec::new();
        let _ = file.read_to_end(&mut contents).await;
        return HttpResponse::Ok().body(contents);
    }

    failed
}

#[inline(always)]
pub async fn osu_add_favourite<'a>(ctx: &Context<'a>) -> HttpResponse {
    let failed = HttpResponse::Unauthorized().body("");
    #[derive(Debug, Deserialize)]
    struct AddFavourites {
        #[serde(rename = "u")]
        username: String,
        #[serde(rename = "h")]
        password_hash: String,
        #[serde(rename = "a")]
        beatmap_set_id: i32,
    }

    // Parse query
    let data = parse_query!(ctx, AddFavourites, failed);
    // Get login
    let player = get_login!(ctx, data, failed);

    let player_id = player.read().await.id;

    if let Ok(_) = ctx
        .database
        .pg
        .execute(
            r#"INSERT INTO "user"."beatmap_collections" (
                "user_id",
                "beatmap_set_id"
             ) VALUES ($1, $2)"#,
            &[&player_id, &data.beatmap_set_id],
        )
        .await
    {
        return HttpResponse::Ok().body("ok");
    };

    HttpResponse::Ok().body("failed")
}

#[inline(always)]
pub async fn osu_get_favourites<'a>(ctx: &Context<'a>) -> HttpResponse {
    let failed = HttpResponse::Unauthorized().body("");
    #[derive(Debug, Deserialize)]
    struct GetFavourites {
        #[serde(rename = "u")]
        username: String,
        #[serde(rename = "h")]
        password_hash: String,
    }

    // Parse query
    let data = parse_query!(ctx, GetFavourites, failed);
    // Get login
    let player = get_login!(ctx, data, failed);

    let player_id = player.read().await.id;

    if let Ok(rows) = ctx
        .database
        .pg
        .query(
            r#"SELECT "beatmap_set_id" FROM "user"."beatmap_collections" WHERE "user_id" = $1"#,
            &[&player_id],
        )
        .await
    {
        let collections: String = rows
            .iter()
            .map(|row| row.get::<'_, _, i32>("beatmap_set_id").to_string() + "\n")
            .collect();
        return HttpResponse::Ok().body(collections);
    };

    failed
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

    // Parse query and get login
    let (data, player) = {
        let mut data = parse_query!(ctx, GetScores, failed);
        // Update game mode with playmod list (rx / ap)
        data.game_mode.update_with_playmod(&data.play_mods.list);
        let player = get_login!(ctx, data, failed);
        (data, player)
    };

    // Try update user stats
    {
        let mut player = player.write().await;
        // Hack detected
        if data.a >= 1 {
            player.hack_detected(data.a, &ctx.database).await;
        }
        // update and get packet
        let user_stats_packet = player.update_mods(&data.game_mode, &data.play_mods).await;
        drop(player);

        // send it
        if let Some(user_stats_packet) = user_stats_packet {
            ctx.player_sessions
                .read()
                .await
                .enqueue_all(&user_stats_packet)
                .await;
        }
    }

    // server is currently not allowed get scores
    if ctx.bancho_config.read().await.all_beatmaps_not_submitted {
        // TODO: send notification to this player once
        return failed;
    }

    // TODO: pp scoreboard or not? get settings from player object
    // TODO: get beatmap by

    // unimplemented("osu-osz2-getscores.php")
    return failed;
}
