use actix_multipart::Multipart;
use actix_web::HttpResponse;
use serde::Deserialize;
use std::time::Instant;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::objects::{Beatmap, ScoreData, ScroeFromDatabase, SubmitModular};
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
    let failed = HttpResponse::Ok().body("error: no");
    let request_ip = match utils::get_realip(&ctx.req).await {
        Ok(ip) => ip,
        Err(_) => {
            error!("[osu_submit_modular] Failed to get submit ip address, refused.");
            return failed;
        }
    };
    debug!(
        "[osu_submit_modular] ip {} wants submit scores..",
        request_ip
    );
    // Check Token
    if !utils::osu_sumit_token_checker(&ctx.req) {
        // TODO: maybe ban someone.
        warn!(
            "[osu_submit_modular] Invalid submit token, ip: {}",
            request_ip
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
                request_ip
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
    let s = match ScoreData::from_submit_modular(&submit_data).await {
        Some(s) => s,
        None => {
            warn!(
                "[osu_submit_modular] Failed to parse score data, ip: {}",
                request_ip
            );
            return failed;
        }
    };
    debug!(
        "[osu_submit_modular] score data parse done, time spent: {:?}",
        score_parse.elapsed()
    );

    // Get login
    let player_name = &s.player_name;
    let player = match ctx
        .bancho
        .player_sessions
        .read()
        .await
        .get_login_by_name(
            player_name,
            &submit_data.password,
            &ctx.global_cache.argon2_cache,
        )
        .await
    {
        Some(p) => p,
        None => {
            warn!("[osu_submit_modular] player {} wants submit score, but not login, wait for next try.", player_name);
            return HttpResponse::Ok().body("try again");
        }
    };
    debug!(
        "[osu_submit_modular] player {} wants submit score, data: {:?}; score: {:?}",
        player_name, submit_data, s
    );

    // Get configs
    let (maintenance_enabled, auto_ban_enabled, auto_ban_whitelist, auto_ban_pp) = {
        let c = ctx.bancho.config.read().await;
        (
            c.maintenance_enabled,
            c.auto_ban_enabled,
            c.auto_ban_whitelist.clone(),
            c.auto_ban_pp(&s.mode),
        )
    };

    let mut player_w = player.write().await;
    let player_id = player_w.id;
    let pp_board = player_w.settings.pp_scoreboard;

    // Update player's active time
    player_w.update_active();

    // Update player's ip data (includes geo ip data)
    if request_ip != player_w.ip {
        player_w.update_ip(request_ip, ctx.geo_db.as_ref());
    }

    // update and get packet
    let user_stats_packet = player_w.update_mods(&s.mode, &s.mods).await;

    // if maintenance_enabled send notification once
    const KEY: &str = "maintenance_score_submit";
    if maintenance_enabled {
        player_w
            .once_notification(
                KEY,
                "Server is currently undergoing maintenance, scores can not be submitted.",
            )
            .await
    };
    drop(player_w);

    // Send it stats
    if let Some(user_stats_packet) = user_stats_packet {
        ctx.bancho
            .player_sessions
            .read()
            .await
            .enqueue_all(&user_stats_packet)
            .await;
    }

    // If maintenance, handle done
    if maintenance_enabled {
        return failed;
    }

    // Get beatmap
    let beatmap = match Beatmap::get(
        Some(&s.beatmap_md5),
        None,
        None,
        &ctx.bancho,
        &ctx.database,
        &ctx.global_cache,
        true,
    )
    .await
    {
        Some(b) => b,
        None => {
            warn!(
                "[osu_submit_modular] Failed to get beatmap, beatmap_md5: {}; player: {}({})",
                s.beatmap_md5, player_name, player_id
            );
            return failed;
        }
    };

    let table = s.mode.full_name();

    // Check duplicate score
    match ctx
        .database
        .pg
        .query(
            &format!(
                "SELECT \"id\" FROM \"game_scores\".\"{}\" WHERE md5 = $1",
                table
            ),
            &[&s.md5],
        )
        .await
    {
        Ok(rows) => {
            if rows.len() > 0 {
                warn!("[osu_submit_modular] Duplicate score detected: player {}({}), score_md5: {}, duplicate with: {:?}", player_name, player_id, s.md5, rows);
                return failed;
            }
        }
        Err(err) => {
            error!(
                "[osu_submit_modular] Failed to check score duplicate, err: {:?}",
                err
            );
        }
    };

    // Play time seconds
    let play_time = if s.pass {
        submit_data.success_time
    } else {
        submit_data.fail_time
    } / 1000;

    // Calculate pp
    let mut calc_failed = false;
    let calc_result = if beatmap.is_ranked() {
        let r = ctx.bancho.pp_calculator.calc(&s.query()).await;
        if r.is_none() {
            warn!(
                "[osu_submit_modular] Failed to calc pp, beatmap md5: {}; player: {}({}); score_data: {:?};",
                s.md5, player_name, player_id, s
            );
            calc_failed = true;
        };
        r
    } else {
        None
    };

    let (pp, raw_pp, stars) = if let Some(r) = &calc_result {
        (
            Some(r.pp),
            r.raw
                .as_ref()
                .map(|raw| serde_json::to_value(raw.clone()).unwrap()),
            Some(r.stars),
        )
    } else {
        (None, None, None)
    };

    // Auto pp ban
    let auto_ban_enabled =
        !auto_ban_whitelist.contains(&player_id) && auto_ban_enabled && auto_ban_pp.is_some();

    // if auto_ban_enabled and pp is calculated, do check
    if auto_ban_enabled {
        if let (Some(calc_result), Some(auto_ban_pp)) = (&calc_result, auto_ban_pp) {
            // Check
            // TODO: if pp >= auto_ban_pp { ban }
            // TODO: send it
        }
    };

    // TODO: Get old personal best
    /* let old_personal_best = match ctx
        .database
        .pg
        .query(
            &format!(
                "SELECT * FROM \"game_scores\".\"{}\" WHERE user_id = $1 AND status = '2'",
                table
            ),
            &[&player_id],
        )
        .await
    {
        Ok(mut rows) => {
            if rows.len() == 0 {
                None
            } else {
                match ScroeFromDatabase::from_row(rows.remove(0)) {
                    Ok(s) => Some(s),
                    Err(err) => {
                        error!(
                            "[osu_submit_modular]: Failed to parse personal_best row, err: {:?}",
                            err
                        );
                        None
                    }
                }
            }
        }
        Err(err) => {
            error!(
                "[osu_submit_modular]: Failed to get personal_best, player {}({}), err: {:?};",
                player_name, player_id, err
            );
            None
        }
    }; */

    // Submit this score
    let query = &format!(
        "INSERT INTO \"game_scores\".\"{}\" (user_id,map_md5,score,pp_v2,pp_v2_raw,stars,accuracy,combo,mods,n300,n100,n50,miss,geki,katu,playtime,perfect,status,grade,client_flags,client_version,\"md5\") VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22) RETURNING \"id\"",
        table
    );
    let score_id: i64 = match ctx
        .database
        .pg
        .query_first(
            &query,
            &[
                &player_id,
                &s.beatmap_md5,
                &s.score,
                &pp,
                &raw_pp,
                &stars,
                &s.accuracy,
                &s.max_combo,
                &(s.mods.value as i32),
                &s.n300,
                &s.n100,
                &s.n50,
                &s.miss,
                &s.geki,
                &s.katu,
                &play_time,
                &s.perfect,
                &s.status.val(),
                &s.grade,
                &s.client_flags,
                &s.osu_version,
                &s.md5,
            ],
        )
        .await
    {
        Ok(row) => row.get("id"),
        Err(err) => {
            error!("[osu_submit_modular] Failed to submit score, err: {:?}; data: {:?}; player: {}({})", err, s, player_name, player_id);
            return failed;
        }
    };

    // If calc failed, add it recalculate task to redis
    if calc_failed {
    // Save replay
    if s.status != SubmissionStatus::Failed {
        if let Some(score_file) = submit_data.score_file {
            match utils::save_replay(
                score_file,
                score_id,
                &ctx.bancho.local_config.data.server.data_dir,
                &s.mode,
            )
            .await
            {
                Ok(_) => debug!(
                    "[osu_submit_modular] Replay file {}({:?}) has saved.",
                    score_id, s.mode
            ),
            Err(err) => error!(
                    "[osu_submit_modular] Failed to save replay {}({:?}), err: {:?}",
                    score_id, s.mode, err
            ),
            };
        } else {
            // TODO: maybe ban someone...
            warn!(
                "[osu_submit_modular] Player {}({}) submitted a score without replay, score: {:?}",
                player_name, player_id, s
            );
        }
    };

    // Create temp talbe
    {
        let score_type = if pp_board { "pp_v2" } else { "score" };
        let score_table = s.mode.full_name();
        let temp_table = format!("{}_{}_{}", score_type, score_table, s.beatmap_md5);

        let start = Instant::now();
        if let Err(err) = ctx
            .database
            .pg
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS \"{0}\"; CREATE TEMP TABLE \"{0}\" AS (
                SELECT ROW_NUMBER() OVER () as rank, res.* FROM (
                    SELECT
                        s.id, u.id as user_id, u.name, u.country, s.{1} AS any_score,
                        s.combo, s.n50, s.n100, s.n300, s.miss,
                        s.katu, s.geki, s.perfect, s.mods, s.create_time
                    FROM game_scores.{2} s
                        LEFT JOIN \"user\".base u ON u.id = s.user_id
                    WHERE s.map_md5 = '{3}'
                        AND s.status = 2
                        AND u.privileges & 1 > 0
                    ORDER BY any_score DESC) AS res);",
                temp_table, score_type, score_table, s.beatmap_md5
            ))
            .await
        {
            error!(
                "[osu_submit_modular]: Failed to create temp table for beatmap {}, err: {:?}",
                s.beatmap_md5, err
            );
        };
        debug!(
            "temp table {} created, time spent: {:?}",
            temp_table,
            start.elapsed()
        );
        ctx.global_cache.cache_temp_table(temp_table).await;
    }

    // TODO: update player's status
    // TODO: fetch old #1
    // TODO: send announce if #1 achieved
    // TODO: save replay
    // TODO: achievements
    // TODO: charts
    // TODO: update beatmap's statistic

    failed
}
