use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::Local;
use peace_database::Database;
use std::time::Instant;

use super::{Caches, ChannelListBuilder, OsuApi};
use crate::settings::{bancho::BanchoConfig, local::LocalConfig};
use crate::utils::lock_wrapper;
use crate::{
    objects::{PPServerApi, PlayerSessions},
    renders::BanchoGet,
    types::ChannelList,
};

pub struct Bancho {
    pub player_sessions: Data<RwLock<PlayerSessions>>,
    pub channel_list: Data<RwLock<ChannelList>>,
    pub osu_api: Data<RwLock<OsuApi>>,
    pub pp_calculator: Data<PPServerApi>,
    pub render_get: Data<RwLock<BanchoGet>>,
    pub config: Data<RwLock<BanchoConfig>>,
    pub local_config: LocalConfig,
}

impl Bancho {
    pub async fn init(local_config: &LocalConfig, database: &Data<Database>) -> Self {
        // Create...
        let config = lock_wrapper(
            BanchoConfig::create(&database)
                .await
                .expect("Failed to create BanchoConfig, could not be initialized."),
        );
        let player_sessions = lock_wrapper(PlayerSessions::new(1000, database));
        let channel_list = lock_wrapper(ChannelListBuilder::new(database, &player_sessions).await);
        let osu_api = lock_wrapper(OsuApi::new(&config).await);
        let pp_calculator = Data::new(PPServerApi::new(&local_config.data));
        let render_get = lock_wrapper(BanchoGet::new(&config).await);

        Bancho {
            player_sessions,
            channel_list,
            osu_api,
            pp_calculator,
            render_get,
            config,
            local_config: local_config.clone(),
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    pub async fn create_score_table(
        beatmap_md5: &str,
        score_table: &str,
        pp_board: bool,
        database: &Database,
        global_cache: &Caches,
        recreate: bool,
    ) -> String {
        let score_type = if pp_board { "pp_v2" } else { "score" };
        let temp_table = format!("{}_{}_{}", score_type, score_table, beatmap_md5);

        // Get scoreboard option 1: Use temporary tables for caching
        // I prefer this option,
        // It may work better when the number of players on the server becomes larger
        // Check temp table cache
        if !recreate {
            let cache_record = global_cache.get_temp_table(&temp_table).await;
            if cache_record.is_some() && (Local::now() - cache_record.unwrap()).num_seconds() < 40 {
                return temp_table;
            };
        };

        // If not temp table exists or its expired, create it
        // TODO: Change to: create table if not exists,
        // according to my design, table will also be created when the score is submitted,
        // so when get scores we may not need to create the table
        let start = Instant::now();
        let first_sql = if recreate {
            format!(
                r#"DROP TABLE IF EXISTS "{0}"; CREATE TEMP TABLE "{0}""#,
                temp_table
            )
        } else {
            format!(r#"CREATE TEMP TABLE IF NOT EXISTS "{0}""#, temp_table)
        };
        if let Err(err) = database
            .pg
            .batch_execute(&format!(
                r#"{0} AS (
    SELECT ROW_NUMBER() OVER () as rank, res.* FROM (
        SELECT
            s.id, u.id as user_id, u.name, u.u_name, u.country, s.pp_v2, s.score,
            s.accuracy, s.combo, s.n50, s.n100, s.n300, s.miss,
            s.katu, s.geki, s.perfect, s.mods, s.create_time
        FROM game_scores.{1} s
            LEFT JOIN "user".base u ON u.id = s.user_id
        WHERE s.map_md5 = '{2}'
            AND s.status = 2
            AND u.privileges & 1 > 0
        ORDER BY {3} DESC) AS res);"#,
                first_sql, score_table, beatmap_md5, score_type
            ))
            .await
        {
            error!(
                "osu_osz2_get_scores: Failed to create temp table for beatmap {}, err: {:?}",
                beatmap_md5, err
            );
        };
        debug!(
            "temp table {} created, time spent: {:?}",
            temp_table,
            start.elapsed()
        );
        global_cache.cache_temp_table(temp_table.clone()).await;
        temp_table
    }
}
