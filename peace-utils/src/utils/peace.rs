use peace_constants::GameMode;
use peace_database::Database;

#[inline(always)]
/// Get player's bp then calculate acc and pp
/// ```
/// calculate_pp_acc(...) -> Option<(acc, pp)>
/// ```
pub async fn player_calculate_pp_acc(
    player_id: i32,
    score_table: &str,
    database: &Database,
) -> Option<(f32, f32)> {
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

    Some((acc, pp_v2))
}

#[inline(always)]
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
            accuracy{1} = $7, 
            pp_v2{1} = $8 
            WHERE "id" = $9"#,
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

#[inline(always)]
pub async fn player_get_pp_acc(
    player_id: i32,
    mode: &GameMode,
    database: &Database,
) -> Option<(f32, f32)> {
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
        Ok(row) => Some((row.try_get("pp").ok()?, row.try_get("acc").ok()?)),
        Err(err) => {
            error!(
                "[player_get_pp_acc] Failed to get player pp and acc from database, err: {:?}",
                err
            );
            None
        }
    }
}
