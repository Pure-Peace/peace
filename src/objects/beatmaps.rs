use crate::{database::Database, types::BeatmapsCache, utils};
use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::{DateTime, Local, Utc};
use tokio_pg_mapper_derive::PostgresMapper;

use super::OsuApi;

macro_rules! fast_from_database {
    ($table:expr, $typ:ty) => {
        impl $typ {
            #[inline(always)]
            pub async fn from_database_by_id(beatmap_id: i32, database: &Database) -> Option<$typ> {
                utils::struct_from_database("beatmaps", $table, "id", &beatmap_id, database).await
            }
            #[inline(always)]
            pub async fn from_database_by_set_id(
                beatmap_set_id: i32,
                database: &Database,
            ) -> Option<$typ> {
                utils::struct_from_database("beatmaps", $table, "set_id", &beatmap_set_id, database)
                    .await
            }
            #[inline(always)]
            pub async fn from_database_by_md5(
                beatmap_md5: &String,
                database: &Database,
            ) -> Option<$typ> {
                utils::struct_from_database("beatmaps", $table, "md5", beatmap_md5, database).await
            }
        }
    };
}

fast_from_database!("maps", BeatmapInfo);
fast_from_database!("stats", BeatmapStats);

#[derive(Debug, Clone)]
pub struct Beatmaps {
    pub info: BeatmapInfo,
    pub stats: Option<BeatmapStats>,
    pub create_time: DateTime<Local>,
}

impl Beatmaps {
    #[inline(always)]
    pub async fn get_from_database(
        beatmap_md5: &String,
        fetch_stats: bool,
        database: &Database,
    ) -> Option<Beatmaps> {
        Some(Beatmaps {
            info: BeatmapInfo::from_database_by_md5(beatmap_md5, database).await?,
            stats: if fetch_stats {
                Some(BeatmapStats::from_database_by_md5(beatmap_md5, database).await?)
            } else {
                None
            },
            create_time: Local::now(),
        })
    }

    #[inline(always)]
    pub async fn get_from_cache(
        beatmap_md5: &String,
        beatmaps_cache: &RwLock<BeatmapsCache>,
    ) -> Option<Beatmaps> {
        beatmaps_cache.read().await.get(beatmap_md5).cloned()
    }

    #[inline(always)]
    pub async fn get_from_osu_api(
        beatmap_md5: &String,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) {
        info!(
            "{:?}",
            osu_api.read().await.fetch_beatmap(beatmap_md5).await
        );
    }
}

#[pg_mapper(table = "beatmaps.maps")]
#[derive(Debug, Clone, PostgresMapper)]
pub struct BeatmapInfo {
    pub server: String,
    pub id: i32,
    pub set_id: i32,
    pub md5: String,
    pub title: String,
    pub artist: String,
    pub diff_name: String,
    pub mapper: String,
    pub mapper_id: i32,
    pub rank_status: i32,
    pub mode: i16,
    pub aim: f32,
    pub spd: f32,
    pub stars: f32,
    pub bpm: f32,
    pub cs: f32,
    pub od: f32,
    pub ar: f32,
    pub hp: f32,
    pub length: i32,
    pub length_drain: i32,
    pub source: Option<String>,
    pub tags: Option<String>,
    pub object_count: i32,
    pub slider_count: i32,
    pub spinner_count: i32,
    pub max_combo: i32,
    pub stars_taiko: f32,
    pub stars_catch: f32,
    pub stars_mania: f32,
    pub fixed_rank_status: bool,
    pub ranked_by: Option<String>,
    pub last_update: Option<String>,
    pub update_time: DateTime<Utc>,
}

#[pg_mapper(table = "beatmaps.stats")]
#[derive(Debug, Clone, PostgresMapper)]
pub struct BeatmapStats {
    pub server: String,
    pub id: i32,
    pub set_id: i32,
    pub md5: String,
    pub plays: i32,
    pub players: i32,
    pub pp: f64,
    pub play_time: i64,
    pub pass: i32,
    pub fail: i32,
    pub clicked: i64,
    pub miss: i64,
    pub pick: i32,
    pub update_time: DateTime<Utc>,
}
