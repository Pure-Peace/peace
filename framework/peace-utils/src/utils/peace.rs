use peace_constants::GameMode;
use peace_database::Database;

pub struct CalcPpAccResult {
    pub pp: f32,
    pub acc: f32,
}

#[inline]
/// Get player's bp then calculate pp and acc
/// ```rust,ignore
/// calculate_pp_acc(...) -> Option<(pp, acc)>
/// ```
pub async fn player_calculate_pp_acc(
    player_id: i32,
    score_table: &str,
    database: &Database,
) -> Option<CalcPpAccResult> {
    // Get bp
    let score_set = match database
        .pg
        .query(
            &format!(
                r#"SELECT s.pp_v2, s.accuracy FROM game_scores.{} s 
                INNER JOIN beatmaps.maps m ON s.map_md5 = m.md5 
                WHERE s.user_id = $1 
                AND s.status = 2 
                AND s.pp_v2 IS NOT NULL
                AND m.rank_status IN (1, 2)
                ORDER BY s.pp_v2 DESC
                LIMIT 100;"#,
                score_table
            ),
            &[&player_id],
        )
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            error!(
                "[player_calculate_pp_acc] Failed to get Player({})'s score set, err: {:?}",
                player_id, err
            );
            return None;
        }
    };
    let score_count = score_set.len();

    // Calc acc from bp
    let acc = if score_count == 1 {
        score_set[0]
            .try_get::<'_, _, f32>("accuracy")
            .unwrap_or(1.0)
    } else {
        let mut total = 0f32;
        let mut div = 0f32;
        for (idx, row) in score_set.iter().enumerate() {
            if let Ok(acc) = row.try_get::<'_, _, f32>("accuracy") {
                let add = (0.95_f32.powi(idx as i32)) * 100.0;
                total += acc * add;
                div += add;
            }
        }
        total / div
    };

    // Calc pp from bp
    let pp_v2 = {
        let mut total = 0f32;
        for (idx, row) in score_set.iter().enumerate() {
            if let Ok(pp) = row.try_get::<'_, _, f32>("pp_v2") {
                total += pp * 0.95_f32.powi(idx as i32);
            }
        }
        total
    };

    Some(CalcPpAccResult { pp: pp_v2, acc })
}

#[inline]
pub async fn player_save_pp_acc(
    player_id: i32,
    mode: &GameMode,
    pp: f32,
    acc: f32,
    database: &Database,
) -> bool {
    match database
        .pg
        .execute(
            &format!(
                r#"UPDATE game_stats.{0} SET 
            accuracy{1} = $1, 
            pp_v2{1} = $2 
            WHERE "id" = $3"#,
                mode.mode_name(),
                mode.sub_mod_table()
            ),
            &[&acc, &pp, &player_id],
        )
        .await
    {
        Ok(_) => true,
        Err(err) => {
            error!(
                "[player_update_pp_acc] Failed to save player stats to database, err: {:?}",
                err
            );
            false
        }
    }
}

#[inline]
pub async fn player_get_pp_acc(
    player_id: i32,
    mode: &GameMode,
    database: &Database,
) -> Option<CalcPpAccResult> {
    match database
        .pg
        .query_first(
            &format!(
                r#"SELECT accuracy{1} as acc, pp_v2{1} as pp FROM game_stats.{0} WHERE "id" = $1"#,
                mode.mode_name(),
                mode.sub_mod_table()
            ),
            &[&player_id],
        )
        .await
    {
        Ok(row) => Some(CalcPpAccResult {
            pp: row.try_get("pp").ok()?,
            acc: row.try_get("acc").ok()?,
        }),
        Err(err) => {
            error!(
                "[player_get_pp_acc] Failed to get player pp and acc from database, err: {:?}",
                err
            );
            None
        }
    }
}

#[inline]
pub async fn pp_recalc_task(
    score_table: &str,
    score_id: i64,
    player_id: i32,
    calc_query: &str,
    database: &Database,
) -> bool {
    let key = format!("calc:{}:{}:{}", score_table, score_id, player_id);
    match database.redis.set(&key, format!("0:{}", calc_query)).await {
        Ok(_) => {
            info!(
                "[osu_submit_modular] set pp-recalculate task to redis, key: {}",
                key
            );
            true
        }
        Err(err) => {
            error!(
                "[osu_submit_modular] Failed to set pp-recalculate task, err: {:?}",
                err
            );
            false
        }
    }
}
