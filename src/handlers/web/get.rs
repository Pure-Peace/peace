#![allow(unused_variables)]
use crate::objects::ScroeFromDatabase;
use std::time::Instant;

use actix_web::{web::Query, HttpResponse};
use async_std::fs::File;
use async_std::prelude::*;
use chrono::Local;
use serde::Deserialize;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    constants::{GameMode, ScoreboardType},
    objects::{Beatmap, PlayMods},
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
                    VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"#,
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
             ) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
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
/// Get scoreboard in-game
///
/// TODO: Performance optimization
/// Get scoreboard option 2: Multiple JOINs
/// ```
///
/// let top_50 = ctx
///     .database
///     .pg
///     .query(
///         &format!(
///             "SELECT ROW_NUMBER() OVER () as rank, res.*
///         FROM (
///             SELECT s.id, u.id as user_id, u.name,
///             s.{0} AS any_score, s.combo,
///             s.n50, s.n100, s.n300, s.miss, s.katu,
///             s.geki, s.perfect, s.mods, s.create_time
///         FROM game_scores.{1} s
///         LEFT JOIN \"user\".base u ON u.id = s.user_id
///         WHERE s.map_md5 = $1
///         AND s.status = 2
///         AND u.privileges & 1 > 0
///         AND s.mods = 0 ORDER BY any_score DESC LIMIT 50) AS res;",
///             score_type, score_table
///         ),
///         &[&beatmap.md5],
///     )
///     .await;
/// let personal_best = ctx
///     .database
///     .pg
///     .query(
///         &format!(
///             "SELECT * FROM (
///         SELECT ROW_NUMBER() OVER (ORDER BY res.any_score DESC) AS rank, res.*
///         FROM (
///             SELECT s.id, u.id as user_id, u.name, s.{0} AS any_score,
///             s.combo, s.n50, s.n100, s.n300, s.miss, s.katu, s.geki, s.perfect,
///             s.mods, s.create_time FROM game_scores.{1} s
///             LEFT JOIN \"user\".base u ON u.id = s.user_id
///             WHERE s.map_md5 = $1
///             AND s.status = 2
///             AND u.privileges & 1 > 0
///             AND s.mods = 0
///         ) AS res) AS foo WHERE user_id = 5;",
///             score_type, score_table
///         ),
///         &[&beatmap.md5],
///     )
///     .await;
/// let total_count = ctx
///     .database
///     .pg
///     .query(
///         &format!(
///             "SELECT COUNT(*) FROM game_scores.{} s
///         LEFT JOIN \"user\".base u ON u.id = s.user_id
///         WHERE map_md5 = $1
///         AND s.status = 2
///         AND u.privileges & 1 > 0
///         AND s.mods = 0;",
///             score_table
///         ),
///         &[&beatmap.md5],
///     )
///     .await;
///
/// ```
pub async fn osu_osz2_get_scores<'a>(ctx: &Context<'a>) -> HttpResponse {
    let not_submit = HttpResponse::Ok().body("-1|false");
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
        file_name: String,
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
        let mut data = parse_query!(ctx, GetScores, not_submit);
        // Update game mode with playmod list (rx / ap / score v2)
        data.game_mode.update_with_playmod(&data.play_mods.list);
        let player = get_login!(ctx, data, not_submit);
        (data, player)
    };

    debug!("osu_osz2_get_scores, data: {:?}", data);
    let (all_beatmaps_not_submitted, all_beatmaps_have_scoreboard) = {
        let c = ctx.bancho.config.read().await;
        (c.all_beatmaps_not_submitted, c.all_beatmaps_have_scoreboard)
    };

    // Player handlers
    // try update user stats, get some info, etc.
    let (pp_board, player_id, player_country) = {
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
        const KEY: &str = "all_beatmaps_not_submitted";
        if all_beatmaps_not_submitted {
            player.once_notification(KEY, "Server is currently not allowed get scores, all beatmaps will display not submitted, please wait for a moment, thanks.").await;
        };

        // Get some info
        let pp_board = player.settings.pp_scoreboard;
        let player_id = player.id;
        let player_country = player.country.clone();

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

        (pp_board, player_id, player_country)
    };

    // server is currently not allowed get scores
    if all_beatmaps_not_submitted {
        return not_submit;
    }

    // get beatmap
    let beatmap = {
        // Try get beatmap with MD5, Setid and filename,
        // from local cache, database cache, and osu!api.
        // if get a beatmap, will auto cache it.
        let b = Beatmap::get(
            Some(&data.beatmap_hash),
            Some(data.beatmap_set_id),
            Some(&data.file_name),
            &ctx.bancho,
            &ctx.database,
            &ctx.global_cache,
            true,
        )
        .await;
        // If cannot get beatmap anyway, return it not submit.
        if b.is_none() {
            return not_submit;
        };
        // for else, we have the beatmap now.
        b.unwrap()
    };

    // Returns it outdated, should update.
    if beatmap.md5 != data.beatmap_hash {
        return HttpResponse::Ok().body("1|false");
    }

    // If unranked and config not allowed
    if !all_beatmaps_have_scoreboard && beatmap.is_unranked() {
        return HttpResponse::Ok().body("0|false");
    };

    let score_type = if pp_board { "pp_v2" } else { "score" };
    let score_table = data.game_mode.full_name();
    let temp_table = format!("{}_{}_{}", score_type, score_table, beatmap.md5);

    // Get scoreboard option 1: Use temporary tables for caching
    // I prefer this option,
    // It may work better when the number of players on the server becomes larger
    // Check temp table cache
    let cache_record = ctx.global_cache.get_temp_table(&temp_table).await;
    if cache_record.is_none() || (Local::now() - cache_record.unwrap()).num_seconds() > 3600 {
        // If not temp table exists or its expired, create it
        // TODO: Change to: create table if not exists,
        // according to my design, table will also be created when the score is submitted,
        // so when get scores we may not need to create the table
        let start = Instant::now();
        if let Err(err) = ctx
            .database
            .pg
            .batch_execute(&format!(
                "CREATE TEMP TABLE IF NOT EXISTS \"{0}\" AS (
    SELECT ROW_NUMBER() OVER () as rank, res.* FROM (
        SELECT
            s.id, u.id as user_id, u.name, u.country, INT4(s.{1}) AS any_score,
            s.combo, s.n50, s.n100, s.n300, s.miss,
            s.katu, s.geki, s.perfect, s.mods, s.create_time
        FROM game_scores.{2} s
            LEFT JOIN \"user\".base u ON u.id = s.user_id
        WHERE s.map_md5 = '{3}'
            AND s.status = 2
            AND u.privileges & 1 > 0
        ORDER BY any_score DESC) AS res);",
                temp_table, score_type, score_table, beatmap.md5
            ))
            .await
        {
            error!(
                "osu_osz2_get_scores: Failed to create temp table for beatmap {}, err: {:?}",
                beatmap.md5, err
            );
        };
        debug!(
            "temp table {} created, time spent: {:?}",
            temp_table,
            start.elapsed()
        );
        ctx.global_cache.cache_temp_table(temp_table.clone()).await;
    };

    // Get scoreboard info
    let start_top_list = Instant::now();
    let top_list = {
        const DEFAULT_WHERE: &'static str = "rank <= 50";
        let mut query = format!("SELECT * FROM \"{}\" WHERE ", temp_table);
        match data.scoreboard_type {
            ScoreboardType::PlayMod => {
                query += &format!("mods = '{}'", data.play_mods.value);
            }
            ScoreboardType::Friends => {
                query += &format!("(user_id IN (SELECT friend_id FROM \"user\".friends WHERE user_id = '{0}') OR user_id = '{0}')", player_id);
            }
            ScoreboardType::Country => {
                query += &format!("country = UPPER('{}')", player_country);
            }
            _ => {
                query += DEFAULT_WHERE;
            }
        };
        // Not default ranking, should re-order
        if !query.contains(DEFAULT_WHERE) {
            query += " ORDER BY any_score DESC LIMIT 50";
        };
        match ctx.database.pg.query_simple(&query).await {
            Ok(scores_row) => {
                let mut scores = Vec::with_capacity(scores_row.len());
                for row in scores_row {
                    match ScroeFromDatabase::from_row(row) {
                        Ok(s) => scores.push(s),
                        Err(err) => error!(
                            "[ScroeFromDatabase]: Failed to parse top_list row, err: {:?}",
                            err
                        ),
                    }
                }
                scores
            }
            Err(err) => {
                error!(
                    "osu_osz2_get_scores: Failed to get top_list scores, query: {}, err: {:?}",
                    query, err
                );
                Vec::new()
            }
        }
    };
    debug!("fetch top-list time spent: {:?}", start_top_list.elapsed());

    let start_personal_best = Instant::now();
    let personal_best = {
        match ctx
            .database
            .pg
            .query_first(
                &format!("SELECT * FROM {} WHERE user_id = $1 LIMIT 1", temp_table),
                &[&player_id],
            )
            .await
        {
            Ok(score_row) => match ScroeFromDatabase::from_row(score_row) {
                Ok(s) => Some(s),
                Err(err) => {
                    error!(
                        "[ScroeFromDatabase]: Failed to parse personal_best row, err: {:?}",
                        err
                    );
                    None
                }
            },
            Err(_) => None,
        }
    };
    debug!(
        "fetch personal_best time spent: {:?}",
        start_personal_best.elapsed()
    );

    let start_scores_count = Instant::now();
    let scores_count_in_table: i64 = {
        match ctx
            .database
            .pg
            .query_first_simple(&format!("SELECT COUNT(*) FROM {}", temp_table))
            .await
        {
            Ok(count_row) => count_row.try_get("count").unwrap_or(0),
            Err(err) => {
                error!(
                    "[ScroeFromDatabase]: Failed to get scores_count_in_table row, err: {:?}",
                    err
                );
                0
            }
        }
    };
    debug!(
        "fetch scores_count time spent: {:?}",
        start_scores_count.elapsed()
    );

    // Build string data return to osu! client
    let start_buid_data = Instant::now();
    // Line 1
    let mut first_line = format!(
        "{rank_status}|{server_has_osz2}|{beatmap_id}|{beatmap_set_id}|{scores_count_in_table}\n",
        rank_status = match beatmap.rank_status_in_server() as i32 {
            // We have unranked scoreboard, so if status is 0(pending), returns it loved(5)
            0 => 5,
            n => n,
        },
        server_has_osz2 = false,
        beatmap_id = beatmap.id,
        beatmap_set_id = beatmap.set_id,
        scores_count_in_table = scores_count_in_table
    );
    // Line 2 - 4
    // {recommend_offset}\n[bold:0,size:20]{beatmap_artist}|{beatmap_title}\n{beatmap_rating}
    first_line += &format!(
        "0\n[bold:0,size:20]{beatmap_artist}|{beatmap_title}\n10.0\n",
        beatmap_artist = beatmap.artist,
        beatmap_title = beatmap.title,
    );
    // Line 5 personal best (optional)
    if let Some(personal_best) = personal_best {
        first_line += &(personal_best.to_string() + "\n");
    } else {
        first_line += "\n";
    };
    // Line 6 - N (top list)
    for s in &top_list {
        first_line += &(s.to_string() + "\n");
    }
    debug!("buid_data time spent: {:?}", start_buid_data.elapsed());

    HttpResponse::Ok().body(first_line)
}
