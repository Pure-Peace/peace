use actix_multipart::Multipart;
use actix_web::HttpResponse;
use chrono::Local;
use peace_objects::beatmaps::Beatmap;
use serde::Deserialize;
use std::time::Instant;

use crate::objects::{Bancho, MiniScore, ScoreData, SubmitModular};
use crate::routes::web::Context;

use peace_constants::SubmissionStatus;

macro_rules! chart_item {
    ($name:expr, $before:expr, $after:expr) => {
        format!(
            "{name}Before:{before}|{name}After:{after}|",
            name = $name,
            before = $before,
            after = $after
        );
    };
}

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
    let request_ip = match peace_utils::web::get_realip(&ctx.req).await {
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
    if !peace_utils::web::osu_sumit_token_checker(&ctx.req) {
        // TODO: maybe ban someone.
        warn!(
            "[osu_submit_modular] Invalid submit token, ip: {}",
            request_ip
        );
        return failed;
    };
    // Parse submit mutipart data
    let submit_parse = Instant::now();
    let submit_data =
        match SubmitModular::from_mutipart(peace_utils::web::get_mutipart_data(payload).await) {
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
    let mut s = match ScoreData::from_submit_modular(&submit_data).await {
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
        .get_login_by_name(player_name, &submit_data.password, &ctx.caches.argon2_cache)
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
    let (maintenance_enabled, auto_ban_enabled, auto_ban_whitelist, auto_ban_pp, front_url) = {
        let cfg_r = ctx.bancho.config.read().await;
        let c = &cfg_r.data;
        (
            c.maintenance.enabled,
            c.auto_ban.enabled,
            c.auto_ban.id_whitelist.clone(),
            c.auto_ban.pp(&s.mode),
            c.server_info.front_url.clone(),
        )
    };

    let mut player_w = player.write().await;
    let player_id = player_w.id;
    let old_stats = player_w.stats.clone();

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
    let beatmap = {
        let expires = ctx.bancho.config.read().await.data.beatmaps.cache_expires;
        let osu_api = ctx.bancho.osu_api.read().await;
        match Beatmap::get(
            Some(&s.beatmap_md5),
            None,
            None,
            None,
            &osu_api,
            &ctx.database,
            true,
            &ctx.caches.beatmap_cache,
            expires,
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
        }
    };

    let score_table = s.mode.full_name();

    // Check duplicate score
    match ctx
        .database
        .pg
        .query(
            &format!(
                r#"SELECT "id" FROM "game_scores"."{}" WHERE md5 = $1"#,
                score_table
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
    let calc_query = s.query();
    let calc_result = {
        let rs = ctx.bancho.pp_calculator.calc(&calc_query).await;
        if rs.is_none() {
            warn!(
                "[osu_submit_modular] Failed to calc pp, beatmap md5: {}; player: {}({}); score_data: {:?};",
                s.md5, player_name, player_id, s
            );
            calc_failed = true;
        };
        rs
    };

    // Try get some value from result
    let (pp, raw_pp, stars) = if let Some(calc_result) = &calc_result {
        // If pp is too small, it does not make sense
        if calc_result.pp < 1.0 {
            (Some(0.0), None, Some(calc_result.stars))
        } else {
            (
                Some(calc_result.pp),
                calc_result
                    .raw
                    .as_ref()
                    .map(|raw| serde_json::to_value(raw.clone()).unwrap()),
                Some(calc_result.stars),
            )
        }
    } else {
        (None, None, None)
    };

    // Auto pp ban handle
    if !calc_failed && calc_result.is_some() {
        let auto_ban_check =
            !auto_ban_whitelist.contains(&player_id) && auto_ban_enabled && auto_ban_pp.is_some();

        // if we should, do check
        if auto_ban_check {
            if let Some(_calc_result) = &calc_result {
                let _auto_ban_pp = auto_ban_pp.unwrap();
                // Check
                // TODO: if pp >= auto_ban_pp { ban }
                // TODO: send it
            }
        };
    };

    let pp_is_best = s.mode.pp_is_best();
    let temp_table = Bancho::create_score_table(
        &s.beatmap_md5,
        &s.mode.full_name(),
        pp_is_best,
        ctx.database,
        ctx.caches,
        false,
    )
    .await;

    // Get old personal best score
    let old_s = MiniScore::from_database(player_id, &temp_table, ctx.database).await;

    // Modify submit status
    if s.pass {
        if let Some(old_s) = &old_s {
            if pp.unwrap_or(0.0) > old_s.pp() {
                s.status = SubmissionStatus::PassedAndTop;
            }
        } else {
            s.status = SubmissionStatus::PassedAndTop;
        };
    }

    // TODO: ...
    if s.status == SubmissionStatus::PassedAndTop {
        let _ = ctx
            .database
            .pg
            .execute(
                &format!(
                    r#"UPDATE game_scores.{} 
            SET status = 1 
            WHERE 
            user_id = $1 
            AND map_md5 = $2 
            AND status = 2"#,
                    score_table
                ),
                &[&player_id, &s.beatmap_md5],
            )
            .await;
    };

    // Submit this score
    let submit_query = format!(
        r#"INSERT INTO "game_scores"."{}" (
            user_id,map_md5,score,pp_v2,pp_v2_raw,
            stars,accuracy,combo,mods,n300,
            n100,n50,miss,geki,katu,
            playtime,perfect,status,grade,client_flags,
            client_version,"md5"
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22) RETURNING "id""#,
        score_table
    );
    let score_id: i64 = match ctx
        .database
        .pg
        .query_first(
            &submit_query,
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
                &(play_time as i32),
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
        Ok(row) => match row.try_get("id") {
            Ok(id) => id,
            Err(err) => {
                error!("[osu_submit_modular] Failed to submit score(without new score id returns), err: {:?}; data: {:?}; player: {}({})", err, s, player_name, player_id);
                return failed;
            }
        },
        Err(err) => {
            error!("[osu_submit_modular] Failed to submit score(database error), err: {:?}; data: {:?}; player: {}({})", err, s, player_name, player_id);
            return failed;
        }
    };

    let mut player_w = player.write().await;

    // Update player's stats
    {
        player_w.update_active();
        player_w.stats.playcount += 1;
        player_w.stats.total_hits += s.total_obj_count(false) - s.miss;
        player_w.stats.total_score += s.score as i64;
        player_w.stats.playtime += play_time;
        if beatmap.is_ranked() {
            player_w.stats.ranked_score += s.score as i64;
        };
        if s.max_combo > player_w.stats.max_combo {
            player_w.stats.max_combo = s.max_combo
        };
        player_w
            .recalculate_stats(&s.mode, ctx.database, !calc_failed)
            .await;
    }

    let new_stats = player_w.stats.clone();
    // If score is passed, and pp calc was failed,
    // we should send notification to user
    if s.pass && calc_failed {
        player_w
            .enqueue(peace_packets::notification(
                "PP calculation fails, Peace will auto recalculate it later.",
            ))
            .await;
    };
    drop(player_w);

    // If calc failed, add it recalculate task to redis
    if calc_failed {
        peace_utils::peace::pp_recalc_task(
            &score_table,
            score_id,
            player_id,
            &calc_query,
            ctx.database,
        )
        .await;
    };

    // Save replay
    if s.status != SubmissionStatus::Failed {
        if let Some(score_file) = submit_data.score_file {
            match peace_utils::bancho::save_replay(
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

    // update beatmap's statistic
    let (play_count, pass_count) = match ctx
        .database
        .pg
        .query_first(
            &format!(
                r#"UPDATE beatmaps.stats SET 
                playcount = playcount + 1,
                play_time = play_time + $1,
                clicked = clicked + $2,
                miss = miss + $3{}{} WHERE "md5" = $4 RETURNING playcount, pass"#,
                if s.pass {
                    ",pass = pass + 1"
                } else {
                    ",fail = fail + 1"
                },
                if submit_data.quit {
                    ",quit = quit + 1"
                } else {
                    ""
                }
            ),
            &[
                &play_time,
                &(&s.total_obj_count(false) - s.miss),
                &s.miss,
                &s.beatmap_md5,
            ],
        )
        .await
    {
        Ok(row) => (
            row.try_get::<'_, _, i32>("playcount").unwrap_or(0),
            row.try_get::<'_, _, i32>("pass").unwrap_or(0),
        ),
        Err(err) => {
            error!(
                "[osu_submit_modular] Failed to update beatmap statistic, err: {:?}, beatmap: {:?}",
                err, beatmap
            );
            (0, 0)
        }
    };

    if s.status == SubmissionStatus::PassedAndTop {
        // TODO: fetch old #1
        // TODO: send announce if #1 achieved
        // Create temp talbe
        let _ = Bancho::create_score_table(
            &s.beatmap_md5,
            &score_table,
            pp_is_best,
            ctx.database,
            ctx.caches,
            true,
        )
        .await;
    } else if s.status == SubmissionStatus::Failed || s.mode.val() > 3 {
        // Submission done, beacuse osu!client will not display chart for these
        return failed;
    }

    // Fetch new score we submitted
    let new_s = MiniScore::from_database(player_id, &temp_table, ctx.database).await;

    // TODO: achievements

    // Charts
    let chart_head = |chart_id: &str, chart_name: &str| {
        format!(
            "chartId:{}|chartUrl:{}/b/{}|chartName:{}|",
            chart_id, front_url, beatmap.id, chart_name
        )
    };

    let chart_beatmap = {
        let body = if let Some(o) = old_s {
            format!(
                "{}{}{}{}{}{}",
                &chart_item!("rank", o.rank, new_s.as_ref().map_or(0, |s| s.rank)),
                &chart_item!("rankedScore", o.score, s.score),
                &chart_item!("totalScore", o.score, s.score),
                &chart_item!("maxCombo", o.combo, s.max_combo),
                &chart_item!("accuracy", o.accuracy, s.calc_acc(false)),
                &chart_item!("pp", o.pp(), pp.unwrap_or(0.0)),
            )
        } else {
            format!(
                "{}{}{}{}{}{}",
                &chart_item!("rank", "", new_s.as_ref().map_or(0, |s| s.rank)),
                &chart_item!("rankedScore", "", s.score),
                &chart_item!("totalScore", "", s.score),
                &chart_item!("maxCombo", "", s.max_combo),
                &chart_item!("accuracy", "", s.calc_acc(false)),
                &chart_item!("pp", "", pp.unwrap_or(0.0)),
            )
        };
        format!(
            "{}{}{}",
            &chart_head("beatmap", "Beatmap Ranking"),
            body,
            &format!("onlineScoreId:{}", score_id),
        )
    };

    let chart_overall = format!(
        "{0}{1}{2}{3}{4}{5}{6}{7}{8}",
        &chart_head("overall", "Overall Ranking"),
        &chart_item!("rank", old_stats.rank, new_stats.rank),
        &chart_item!(
            "rankedScore",
            old_stats.ranked_score,
            new_stats.ranked_score
        ),
        &chart_item!("totalScore", old_stats.total_score, new_stats.total_score),
        &chart_item!("maxCombo", old_stats.max_combo, new_stats.max_combo),
        &chart_item!("accuracy", old_stats.accuracy, new_stats.accuracy),
        &chart_item!("pp", old_stats.pp_v2, new_stats.pp_v2),
        // TODO: achievements
        "achievements-new:",
        &format!("|onlineScoreId:{}", score_id),
    );

    let charts = format!(
        "beatmapId:{0}|beatmapSetId:{1}|beatmapPlaycount:{2}|beatmapPasscount:{3}|approvedDate:{4}\n{5}\n{6}",
        beatmap.id,
        beatmap.set_id,
        play_count,
        pass_count,
        beatmap.last_update.unwrap_or(Local::now()).format("%Y-%m-%d %H:%M:%S").to_string(),
        chart_beatmap,
        chart_overall
    );

    HttpResponse::Ok().body(charts)
}
