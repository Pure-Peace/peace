use ntex::web::types::Data;
use tokio::sync::RwLock;
use chrono::Local;
use hashbrown::HashMap;
use std::time::Instant;

use peace_database::Database;
use peace_objects::{osu_api::OsuApi, pp_server_api::PPServerApi};
use peace_settings::{bancho::BanchoConfig, local::LocalConfig};

use super::{Caches, ChannelListBuilder};
use crate::{
    objects::PlayerSessions,
    renders::BanchoGet,
    types::{ChannelList, MatchList},
};

pub struct Bancho {
    pub player_sessions: Data<RwLock<PlayerSessions>>,
    pub channel_list: Data<RwLock<ChannelList>>,
    pub match_list: Data<RwLock<MatchList>>,
    pub osu_api: Data<RwLock<OsuApi>>,
    pub pp_calculator: Data<PPServerApi>,
    pub render_get: Data<RwLock<BanchoGet>>,
    pub config: Data<RwLock<BanchoConfig>>,
    pub local_config: LocalConfig,
}

impl Bancho {
    pub async fn init(cfg: &LocalConfig, database: &Data<Database>) -> Self {
        use peace_utils::web::lock_wrapper as lw;
        // Create...
        let config = BanchoConfig::create(&database)
            .await
            .expect("Failed to create BanchoConfig, could not be initialized.");
        let osu_api_keys = config.data.server.osu_api_keys.clone();

        let config = lw(config);
        let player_sessions = lw(PlayerSessions::new(1000, database));
        let channel_list = lw(ChannelListBuilder::channels_from_database(database).await);
        let match_list = lw(HashMap::with_capacity(50));
        let osu_api = lw(OsuApi::new(osu_api_keys).await);
        let pp_calculator = Data::new(PPServerApi::new(
            cfg.data.pp_server.url.clone(),
            cfg.data.pp_server.pp_calc_timeout,
        ));
        let render_get = lw(BanchoGet::new(&config).await);

        Bancho {
            player_sessions,
            channel_list,
            match_list,
            osu_api,
            pp_calculator,
            render_get,
            config,
            local_config: cfg.clone(),
        }
    }

    #[inline]
    pub async fn create_score_table(
        beatmap_md5: &str,
        score_table: &str,
        pp_board: bool,
        database: &Database,
        caches: &Caches,
        recreate: bool,
    ) -> String {
        let score_type = if pp_board { "pp_v2" } else { "score" };
        let temp_table = format!("{}_{}_{}", score_type, score_table, beatmap_md5);

        // Get scoreboard option 1: Use temporary tables for caching
        // I prefer this option,
        // It may work better when the number of players on the server becomes larger
        // Check temp table cache
        if !recreate {
            let cache_record = caches.get_temp_table(&temp_table).await;
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
        caches.cache_temp_table(temp_table.clone()).await;
        temp_table
    }
}
