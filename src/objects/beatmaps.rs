use std::time::Instant;

use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use field_names::FieldNames;
use postgres_types::{FromSql, ToSql};
use serde::Deserialize;
use serde_str;
use tokio_pg_mapper_derive::PostgresMapper;

use super::{Bancho, Caches, OsuApi};
use crate::utils::{from_str_bool, from_str_optional};
use crate::{database::Database, types::BeatmapsCache, utils};

macro_rules! fast_from_database {
    ($table:expr, $typ:ty) => {
        impl $typ {
            #[inline(always)]
            pub async fn from_database_by_id(beatmap_id: i32, database: &Database) -> Option<$typ> {
                utils::struct_from_database(
                    "beatmaps",
                    $table,
                    "id",
                    <$typ>::get_query_fields().as_str(),
                    &beatmap_id,
                    database,
                )
                .await
            }
            #[inline(always)]
            pub async fn from_database_by_set_id(
                beatmap_set_id: i32,
                database: &Database,
            ) -> Option<$typ> {
                utils::struct_from_database(
                    "beatmaps",
                    $table,
                    "set_id",
                    <$typ>::get_query_fields().as_str(),
                    &beatmap_set_id,
                    database,
                )
                .await
            }
            #[inline(always)]
            pub async fn from_database_by_md5(
                beatmap_md5: &String,
                database: &Database,
            ) -> Option<$typ> {
                utils::struct_from_database(
                    "beatmaps",
                    $table,
                    "md5",
                    <$typ>::get_query_fields().as_str(),
                    beatmap_md5,
                    database,
                )
                .await
            }
        }
    };
}

fast_from_database!("maps", BeatmapInfo);
fast_from_database!("stats", BeatmapStats);

#[derive(Debug, Clone)]
pub struct Beatmaps {
    pub info: Option<BeatmapInfo>,
    pub stats: Option<BeatmapStats>,
    pub create_time: DateTime<Local>,
}

impl Beatmaps {
    #[inline(always)]
    pub fn default() -> Self {
        Beatmaps {
            info: None,
            stats: None,
            create_time: Local::now(),
        }
    }

    #[inline(always)]
    pub fn is_not_submit(&self) -> bool {
        self.info.is_none()
    }

    #[inline(always)]
    pub async fn get(
        md5: &String,
        bancho: &Bancho,
        database: &Database,
        cache: &Caches,
        use_cache: bool,
    ) -> Option<Self> {
        if use_cache {
            // Try get beatmap from local cache
            let expire = bancho.config.read().await.timeout_beatmap_cache;
            if let Some(b) = Self::get_from_cache(&md5, &cache.beatmaps_cache, expire).await {
                info!("[Beatmaps] get from cache: {};", md5);
                return Some(b);
            }
        }

        // Try get from database or api, etc.
        match Self::get_from_source(md5, bancho, database).await {
            Ok(b) => {
                cache
                    .beatmaps_cache
                    .write()
                    .await
                    .insert(md5.to_string(), b.clone());
                Some(b)
            }
            Err(i) => {
                // 0: not exists; -1: request err; -2: parse err
                if i != -1 {
                    cache
                        .beatmaps_cache
                        .write()
                        .await
                        .insert(md5.to_string(), Beatmaps::default());
                };
                info!(
                    "[Beatmaps:err{}] Failed to get beatmap from anyway: {}, cache it not submitted.",i, md5
                );
                None
            }
        }
    }

    #[inline(always)]
    pub async fn get_from_source(
        md5: &String,
        bancho: &Bancho,
        database: &Database,
    ) -> Result<Self, i32> {
        let start = Instant::now();
        // Try get beatmap from database
        if let Some(b) = Self::get_from_database(&md5, false, &database).await {
            info!(
                "[Beatmaps] get from database: {}; time spent: {:?}",
                md5,
                start.elapsed()
            );
            return Ok(b);
        }
        match Self::get_from_osu_api(&md5, &bancho.osu_api, &database).await {
            Ok(b) => {
                info!(
                    "[Beatmaps] get from osu!api: {}; time spent: {:?}",
                    md5,
                    start.elapsed()
                );
                Ok(b)
            }
            Err(i) => Err(i),
        }
    }

    #[inline(always)]
    pub async fn get_from_database(
        beatmap_md5: &String,
        fetch_stats: bool,
        database: &Database,
    ) -> Option<Self> {
        Some(Self {
            info: Some(BeatmapInfo::from_database_by_md5(beatmap_md5, database).await?),
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
    pub fn is_expired(&self, expires: i64) -> bool {
        if let Some(info) = &self.info {
            // Fixed never expire!
            if info.fixed_rank_status {
                return false;
            }
        };
        (Local::now() - self.create_time).num_seconds() > expires
    }

    #[inline(always)]
    pub async fn get_from_cache(
        beatmap_md5: &String,
        beatmaps_cache: &RwLock<BeatmapsCache>,
        expire: i64,
    ) -> Option<Self> {
        debug!("try get beatmap {} info from cache...", beatmap_md5);
        match beatmaps_cache.read().await.get(beatmap_md5) {
            Some(b) => {
                // Check is expired
                if b.is_expired(expire) {
                    debug!("get from beatmap cache but expired: {}", beatmap_md5);
                    return None;
                }
                Some(b.clone())
            }
            None => None,
        }
    }

    #[inline(always)]
    pub async fn get_from_osu_api(
        beatmap_md5: &String,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) -> Result<Self, i32> {
        match BeatmapFromApi::get_from_osu_api(beatmap_md5, osu_api, database).await {
            Ok(f) => {
                debug!("get it, do Beatmaps convert from api object...");
                Ok(f.convert_to_beatmap())
            }
            Err(i) => Err(i),
        }
    }
}

#[derive(Debug, FieldNames, ToSql, Deserialize, Clone)]
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
    #[serde(deserialize_with = "from_str_optional")]
    max_combo: Option<i32>,
    #[serde(rename = "total_length", with = "serde_str")]
    length: i32,
    #[serde(rename = "hit_length", with = "serde_str")]
    length_drain: i32,
    #[serde(rename = "diff_aim", deserialize_with = "from_str_optional")]
    aim: Option<f32>,
    #[serde(rename = "diff_speed", deserialize_with = "from_str_optional")]
    spd: Option<f32>,
    #[serde(rename = "difficultyrating", with = "serde_str")]
    stars: f32,
    #[serde(rename = "submit_date", with = "my_serde")]
    submit_time: Option<DateTime<Local>>,
    #[serde(rename = "approved_date", with = "my_serde")]
    approved_time: Option<DateTime<Local>>,
    #[serde(with = "my_serde")]
    last_update: Option<DateTime<Local>>,
}

impl BeatmapFromApi {
    #[inline(always)]
    pub fn convert_to_beatmap(self) -> Beatmaps {
        Beatmaps {
            info: Some(BeatmapInfo::from(self)),
            stats: None,
            create_time: Local::now(),
        }
    }

    #[inline(always)]
    pub async fn get_from_osu_api(
        beatmap_md5: &String,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) -> Result<Self, i32> {
        debug!("try get beatmap {} info from osu!api...", beatmap_md5);
        match osu_api.read().await.fetch_beatmap_by_md5(beatmap_md5).await {
            Ok(beatmap_from_api) => {
                debug!("get_from_osu_api success: {:?}", beatmap_from_api);
                beatmap_from_api.save_to_database(database).await;
                Ok(beatmap_from_api)
            }
            Err(i) => Err(i),
        }
    }

    #[inline(always)]
    pub async fn save_to_database(&self, database: &Database) -> Option<()> {
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
                    &self.id,
                    &self.set_id,
                    &self.md5,
                    &self.artist,
                    &self.artist_unicode,
                    &self.title,
                    &self.title_unicode,
                    &self.mapper,
                    &self.mapper_id,
                    &self.rank_status,
                    &self.diff_name,
                    &self.cs,
                    &self.od,
                    &self.ar,
                    &self.hp,
                    &self.mode,
                    &self.object_count,
                    &self.slider_count,
                    &self.spinner_count,
                    &self.bpm,
                    &self.source,
                    &self.tags,
                    &self.genre_id,
                    &self.language_id,
                    &self.storyboard,
                    &self.video,
                    &self.max_combo,
                    &self.length,
                    &self.length_drain,
                    &self.aim,
                    &self.spd,
                    &self.stars,
                    &self.submit_time,
                    &self.approved_time,
                    &self.last_update,
                ],
            )
            .await
        {
            Ok(_) => {
                debug!("successfully insert beatmap {} to database.", self.md5);
                Some(())
            }
            Err(err) => {
                error!(
                    "failed to insert beatmap {} to database, err: {:?}",
                    self.md5, err
                );
                None
            }
        }
    }
}

#[pg_mapper(table = "beatmaps.maps")]
#[derive(Debug, FieldNames, Clone, FromSql, ToSql, PostgresMapper)]
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
    pub length: i32,
    pub length_drain: i32,
    pub max_combo: Option<i32>,
    pub fixed_rank_status: bool,
    pub ranked_by: Option<String>,
    pub last_update: Option<DateTime<Local>>,
    pub update_time: DateTime<Local>,
}

impl BeatmapInfo {
    #[inline]
    pub fn get_query_fields() -> String {
        format!("(\"{}\")", BeatmapInfo::FIELDS.join("\",\""))
    }
}

impl From<BeatmapFromApi> for BeatmapInfo {
    fn from(f: BeatmapFromApi) -> Self {
        Self {
            server: "ppy".to_string(),
            id: f.id,
            set_id: f.set_id,
            md5: f.md5,
            title: f.title,
            artist: f.artist,
            diff_name: f.diff_name,
            mapper: f.mapper,
            mapper_id: f.mapper_id,
            rank_status: f.rank_status,
            mode: f.mode,
            length: f.length,
            length_drain: f.length_drain,
            max_combo: f.max_combo,
            fixed_rank_status: [1, 2].contains(&f.rank_status),
            ranked_by: None,
            last_update: f.last_update,
            update_time: f.last_update.unwrap_or(Local::now()),
        }
    }
}

#[pg_mapper(table = "beatmaps.stats")]
#[derive(Debug, FieldNames, Clone, PostgresMapper)]
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
    pub update_time: DateTime<Local>,
}

impl BeatmapStats {
    #[inline]
    pub fn get_query_fields() -> String {
        format!("(\"{}\")", BeatmapStats::FIELDS.join("\",\""))
    }
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
