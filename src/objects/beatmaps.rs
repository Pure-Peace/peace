use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use field_names::FieldNames;
use serde::{Deserialize, Serialize};
use serde_str;

use postgres_types::{FromSql, ToSql};

use chrono::Utc;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::utils::from_str_bool;
use crate::{database::Database, types::BeatmapsCache, utils};

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
                let stats = BeatmapStats::from_database_by_md5(beatmap_md5, database).await;
                if let Some(s) = stats {
                    Some(s)
                } else {
                    None
                }
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
        debug!("try get beatmap {} info from cache...", beatmap_md5);
        beatmaps_cache.read().await.get(beatmap_md5).cloned()
    }

    #[inline(always)]
    pub async fn get_from_osu_api(
        beatmap_md5: &String,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) -> Option<BeatmapFromApi> {
        debug!("try get beatmap {} info from osu!api...", beatmap_md5);
        match osu_api.read().await.fetch_beatmap_by_md5(beatmap_md5).await {
            Some(f) => {
                debug!("get_from_osu_api success: {:?}", f);
                match database
                    .pg
                    .execute(
                        format!(
                            "INSERT INTO \"beatmaps\".\"maps\" (\"{}\") VALUES ({})",
                            BeatmapFromApi::FIELDS.join("\",\""),
                            utils::build_s(BeatmapFromApi::FIELDS.len())
                        )
                        .as_str(),
                        &[
                            &f.id,
                            &f.set_id,
                            &f.md5,
                            &f.artist,
                            &f.artist_unicode,
                            &f.title,
                            &f.title_unicode,
                            &f.mapper,
                            &f.mapper_id,
                            &f.rank_status,
                            &f.diff_name,
                            &f.cs,
                            &f.od,
                            &f.ar,
                            &f.hp,
                            &f.mode,
                            &f.object_count,
                            &f.slider_count,
                            &f.spinner_count,
                            &f.bpm,
                            &f.source,
                            &f.tags,
                            &f.genre_id,
                            &f.language_id,
                            &f.storyboard,
                            &f.video,
                            &f.max_combo,
                            &f.length,
                            &f.length_drain,
                            &f.aim,
                            &f.spd,
                            &f.stars,
                            &f.submit_time,
                            &f.approved_time,
                            &f.last_update,
                        ],
                    )
                    .await
                {
                    Ok(_) => {
                        debug!("successfully insert beatmap {} to database.", beatmap_md5);
                    }
                    Err(err) => {
                        debug!(
                            "failed to insert beatmap {} to database, err: {:?}",
                            beatmap_md5, err
                        );
                    }
                };
                Some(f)
            }
            None => None,
        }
    }
}

#[pg_mapper(table = "beatmaps.maps")]
#[derive(Debug, Clone, FromSql, ToSql, PostgresMapper)]
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
    pub last_update: Option<DateTime<Local>>,
    pub update_time: DateTime<Local>,
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

#[derive(Debug, FieldNames, ToSql, Deserialize, Serialize, Clone)]
pub struct BeatmapFromApi {
    #[serde(rename = "beatmap_id", with = "serde_str")]
    id: i32,
    #[serde(rename = "beatmapset_id", with = "serde_str")]
    set_id: i32,
    #[serde(rename = "file_md5")]
    md5: String,
    artist: String,
    artist_unicode: Option<String>,
    title: String,
    title_unicode: Option<String>,
    #[serde(rename = "creator")]
    mapper: String,
    #[serde(rename = "creator_id", with = "serde_str")]
    mapper_id: i32,
    #[serde(rename = "approved", with = "serde_str")]
    rank_status: i32,
    #[serde(rename = "version")]
    diff_name: String,
    #[serde(rename = "diff_size", with = "serde_str")]
    cs: f32,
    #[serde(rename = "diff_overall", with = "serde_str")]
    od: f32,
    #[serde(rename = "diff_approach", with = "serde_str")]
    ar: f32,
    #[serde(rename = "diff_drain", with = "serde_str")]
    hp: f32,
    #[serde(with = "serde_str")]
    mode: i16,
    #[serde(rename = "count_normal", with = "serde_str")]
    object_count: i32,
    #[serde(rename = "count_slider", with = "serde_str")]
    slider_count: i32,
    #[serde(rename = "count_spinner", with = "serde_str")]
    spinner_count: i32,
    #[serde(with = "serde_str")]
    bpm: f32,
    source: Option<String>,
    tags: Option<String>,
    #[serde(with = "serde_str")]
    genre_id: i16,
    #[serde(with = "serde_str")]
    language_id: i16,
    #[serde(deserialize_with = "from_str_bool")]
    storyboard: bool,
    #[serde(deserialize_with = "from_str_bool")]
    video: bool,
    #[serde(with = "serde_str")]
    max_combo: i32,
    #[serde(rename = "total_length", with = "serde_str")]
    length: i32,
    #[serde(rename = "hit_length", with = "serde_str")]
    length_drain: i32,
    #[serde(rename = "diff_aim", with = "serde_str")]
    aim: f32,
    #[serde(rename = "diff_speed", with = "serde_str")]
    spd: f32,
    #[serde(rename = "difficultyrating", with = "serde_str")]
    stars: f32,
    #[serde(rename = "submit_date", with = "my_serde")]
    submit_time: Option<DateTime<Local>>,
    #[serde(rename = "approved_date", with = "my_serde")]
    approved_time: Option<DateTime<Local>>,
    #[serde(with = "my_serde")]
    last_update: Option<DateTime<Local>>,
}

mod my_serde {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Option<DateTime<Local>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.unwrap().format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer);
        if s.is_err() {
            return Ok(None);
        };
        match Local.datetime_from_str(&s.unwrap(), FORMAT) {
            Ok(t) => Ok(Some(t)),
            Err(_err) => Ok(None),
        }
    }
}
