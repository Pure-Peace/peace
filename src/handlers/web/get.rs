#![allow(unused_variables)]
use actix_web::{web::Query, HttpResponse};
use async_std::fs::File;
use async_std::prelude::*;
use serde::Deserialize;

use crate::{
    constants::{GameMode, ScoreboardType},
    objects::{Beatmaps, PlayMods},
    packets,
    routes::web::Context,
    utils,
};

macro_rules! get_login {
    ($ctx:ident, $data:ident, $failed:ident) => {
        match $ctx
            .bancho
            .player_sessions
            .read()
            .await
            .get_login_by_name(
                &$data.username,
                &$data.password_hash,
                &$ctx.global_cache.argon2_cache,
            )
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

const HAX_DETECTED_NOTIFICATION: Option<&str> = Some(
    "Warning: Please do not cheat. It makes no sense. If you do, you will most likely be banned.",
);

#[inline(always)]
pub async fn check_updates<'a>(ctx: &Context<'a>) -> HttpResponse {
    let default = HttpResponse::Ok().body("[]");
    const ACTION_VALID: &[&str; 3] = &["check", "path", "error"];
    const STREAM_VALID: &[&str; 4] = &["cuttingedge", "stable40", "beta40", "stable"];
    lazy_static::lazy_static! {
        static ref REQ_CLIENT: reqwest::Client = reqwest::Client::new();
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
    struct CheckUpdates {
        action: String,
        stream: String,
    }

    // Read config
    let (update_enabled, update_expires) = {
        let temp = ctx.bancho.config.read().await;
        (temp.client_update_enabled, temp.client_update_expires)
    };

    // Check should we update?
    if !update_enabled {
        return default;
    }

    let data = parse_query!(ctx, CheckUpdates, default);

    // Validation check
    if !ACTION_VALID.contains(&data.action.as_str())
        || !STREAM_VALID.contains(&data.stream.as_str())
    {
        return default;
    }
    let query = format!("action={}&stream={}", data.action, data.stream);

    // Try get from redis
    if let Ok(result) = ctx.database.redis.get::<String, _>(&query).await {
        debug!(
            "check_updates: cache hitted, query: {}, result: {:?};",
            query,
            result.get(1..30)
        );
        return HttpResponse::Ok().body(result);
    };

    // Try get from osu! web
    let resp = REQ_CLIENT
        .get(&("https://old.ppy.sh/web/check-updates.php?".to_string() + &query))
        .send()
        .await;

    if let Err(err) = resp {
        warn!(
            "(1) failed to request osu! check-updates with query: {}, err: {:?}",
            query, err
        );
        return default;
    };

    let resp = resp.unwrap();
    if resp.status() != 200 {
        warn!(
            "(2) failed to request osu! check-updates with query: {}, err: {:?}",
            query,
            resp.text().await
        );
        return default;
    }

    let resp = resp.text().await;
    if let Err(err) = resp {
        warn!(
            "(3) failed to request osu! check-updates with query: {}, err: {:?}",
            query, err
        );
        return default;
    }

    // Store to redis
    let resp: String = resp.unwrap();
    let mut cmd = ctx.database.redis.cmd("SET");
    if let Err(err) = ctx
        .database
        .redis
        .execute_cmd(
            &cmd.arg(query)
                .arg(resp.clone())
                .arg("EX")
                .arg(update_expires),
        )
        .await
    {
        warn!(
            "failed store check-updates result: {:?}, err: {:?}",
            resp.get(1..30),
            err
        );
    }

    HttpResponse::Ok().body(resp)
}

#[inline(always)]
pub async fn bancho_connect<'a>(_ctx: &Context<'a>) -> HttpResponse {
    HttpResponse::Ok().body("")
}

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
            HAX_DETECTED_NOTIFICATION,
            &ctx.database,
        )
        .await;
    // TODO: may sent to discord hooks

    done
}

#[inline(always)]
pub async fn osu_rate<'a>(ctx: &Context<'a>) -> HttpResponse {
    let failed = HttpResponse::Unauthorized().body("");
    #[derive(Debug, Deserialize)]
    struct Rating {
        #[serde(rename = "u")]
        username: String,
        #[serde(rename = "p")]
        password_hash: String,
        #[serde(rename = "c")]
        beatmap_md5: String,
        #[serde(rename = "v")]
        vote: Option<i16>,
    }
    let data = parse_query!(ctx, Rating, failed);
    let player = get_login!(ctx, data, failed);

    let (player_id, player_name) = {
        let p = player.read().await;
        (p.id, p.name.clone())
    };

    // Data is include vote?
    if let Some(vote) = data.vote {
        // limit rating value in 1 - 10
        let vote = match vote {
            v if v > 10 => 10,
            v if v < 1 => 1,
            v => v,
        };

        // Add it
        match ctx
            .database
            .pg
            .execute(
                r#"INSERT INTO "beatmaps"."ratings" 
                        ("user_id","map_md5","rating") 
                    VALUES ($1, $2, $3)"#,
                &[&player_id, &player_name, &vote],
            )
            .await
        {
            Ok(_) => {
                debug!(
                    "player {}({}) voted beatmap {}, rating: {}",
                    player_name, player_id, data.beatmap_md5, vote
                );
            }
            Err(err) => {
                error!(
                    "failed to add ratings to beatmap {} by player {}({}), voted: {}; err: {:?}",
                    data.beatmap_md5, player_name, player_id, vote, err
                );
            }
        };
    } else {
        // Check is already voted?
        if let Err(_) = ctx
            .database
            .pg
            .query_first(
                r#"SELECT 1 FROM "beatmaps"."ratings" WHERE "user_id" = $1 AND "map_md5" = $2"#,
                &[&player_id, &data.beatmap_md5],
            )
            .await
        {
            // Not already, player can vote.
            return HttpResponse::Ok().body("ok");
        };
    }

    let value = utils::get_beatmap_rating(&data.beatmap_md5, &ctx.database).await;
    HttpResponse::Ok().body(format!("alreadyvoted\n{}", value.unwrap_or(10.0)))
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
    if let Some(background_images) = &ctx.bancho.config.read().await.seasonal_backgrounds {
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

    debug!("osu_osz2_get_scores, data: {:?}", data);
    let all_beatmaps_not_submitted = ctx.bancho.config.read().await.all_beatmaps_not_submitted;

    // Player handlers
    // try update user stats, get some info, etc.
    let pp_board = {
        let mut player = player.write().await;

        // Hack detected
        if data.a > 0 {
            player
                .hack_detected(
                    data.a,
                    "osz2_get_scores",
                    HAX_DETECTED_NOTIFICATION,
                    &ctx.database,
                )
                .await;
            // TODO: may sent to discord hooks
        }

        // update and get packet
        let user_stats_packet = player.update_mods(&data.game_mode, &data.play_mods).await;

        // if all_beatmaps_not_submitted handle
        if all_beatmaps_not_submitted {
            const KEY: &str = "all_beatmaps_not_submitted";
            // send notification to this player once
            if !player.flag_cache.contains_key(KEY) {
                player.enqueue(packets::notification("Server is currently not allowed get scores, all beatmaps will display not submitted, please wait for a moment, thanks.")).await;
                player.flag_cache.insert(KEY.to_owned(), None);
            }
        }

        // Get some info
        let pp_board = player.settings.pp_scoreboard;

        // Release player write lock
        drop(player);

        // send it stats
        if let Some(user_stats_packet) = user_stats_packet {
            ctx.bancho
                .player_sessions
                .read()
                .await
                .enqueue_all(&user_stats_packet)
                .await;
        }

        pp_board
    };

    // server is currently not allowed get scores
    if all_beatmaps_not_submitted {
        return failed;
    }

    // Get beatmap
    let beatmap = Beatmaps::get(
        &data.beatmap_hash,
        &ctx.bancho,
        &ctx.database,
        &ctx.global_cache,
        true,
    )
    .await;
    if beatmap.is_none() {
        return failed;
    }
    let beatmap = beatmap.unwrap();

    // TODO: XXX

    // unimplemented("osu-osz2-getscores.php")
    return failed;
}
