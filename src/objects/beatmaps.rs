use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use field_names::FieldNames;
use postgres_types::{FromSql, ToSql};
use serde::Deserialize;
use serde_str;
use std::any::Any;
use std::fmt::Display;
use tokio_pg_mapper_derive::PostgresMapper;

use super::{Bancho, Caches, OsuApi};
use crate::utils::{from_str_bool, from_str_optional};
use crate::{database::Database, utils};

#[derive(Debug)]
pub enum GetBeatmapMethod {
    Md5,
    Bid,
    Sid,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ApiError {
    NotExists,
    RequestError,
    ParseError,
}

#[derive(Debug, Clone)]
pub struct Beatmaps {
    pub info: Option<BeatmapInfo>,
    pub create_time: DateTime<Local>,
}

impl Beatmaps {
    #[inline(always)]
    pub fn default() -> Self {
        Beatmaps {
            info: None,
            create_time: Local::now(),
        }
    }

    #[inline(always)]
    pub fn is_not_submit(&self) -> bool {
        self.info.is_none()
    }

    #[inline(always)]
    pub async fn from_database<T: Any + Display>(
        key: &T,
        method: GetBeatmapMethod,
        database: &Database,
    ) -> Option<Self> {
        let v = key as &dyn Any;
        let info = match method {
            GetBeatmapMethod::Md5 => {
                BeatmapInfo::from_database_by_md5(v.downcast_ref().unwrap(), database).await?
            }
            GetBeatmapMethod::Sid => {
                BeatmapInfo::from_database_by_sid(*v.downcast_ref().unwrap(), database).await?
            }
            GetBeatmapMethod::Bid => {
                BeatmapInfo::from_database_by_bid(*v.downcast_ref().unwrap(), database).await?
            }
        };
        let create_time = info.update_time.clone();
        let new = Self {
            info: Some(info),
            create_time,
        };
        info!(
            "[Beatmaps] get from database with Method({:?}): {}",
            method, key
        );
        Some(new)
    }

    #[inline(always)]
    pub async fn from_osu_api<T: Any + Display>(
        key: &T,
        method: GetBeatmapMethod,
        file_name: Option<&String>,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) -> Result<Self, ApiError> {
        Ok(
            BeatmapFromApi::from_osu_api(key, method, file_name, osu_api, database)
                .await?
                .convert_to_beatmap(),
        )
    }

    #[inline(always)]
    /// Get beatmap with MD5 or SID
    pub async fn get(
        md5: Option<&String>,
        sid: Option<i32>,
        file_name: Option<&String>,
        bancho: &Bancho,
        database: &Database,
        cache: &Caches,
        try_from_cache: bool,
    ) -> Option<Self> {
        let expire = bancho.config.read().await.timeout_beatmap_cache;
        // MD5 Available
        if let Some(md5) = md5 {
            // Try get beatmap from local cache
            if try_from_cache {
                if let Some(b) = Self::from_cache(md5, &cache, expire).await {
                    info!("[Beatmaps] Get from cache: {};", md5);
                    return Some(b);
                };
            };

            // Local cache expired or not founded, then
            // Try get beatmap from database
            if let Some(b) = Beatmaps::from_database(md5, GetBeatmapMethod::Md5, database).await {
                // If not expired, cache it locally and returns it.
                if !b.is_expired(expire) {
                    cache.cache_beatmap(md5.clone(), &b).await;
                    return Some(b);
                }
            };

            // Database cache expired or not founded, then
            // Try get beatmap from osu!api (try with md5)
            match Beatmaps::from_osu_api(
                md5,
                GetBeatmapMethod::Md5,
                None,
                &bancho.osu_api,
                database,
            )
            .await
            {
                Ok(b) => {
                    cache.cache_beatmap(md5.clone(), &b).await;
                    return Some(b);
                }
                Err(err) => {
                    // If request error, we will not cache it.
                    debug!(
                        "[Beatmaps] Failed to get beatmap ({}), err: {:?};",
                        md5, err
                    );
                    if err != ApiError::RequestError {
                        // Else, cache it Not submitted
                        cache.cache_beatmap(md5.clone(), &Beatmaps::default()).await;
                    };
                }
            };
        };

        // Cannot get from osu!api from md5, then
        // If SID Available,
        // Try get beatmap from osu!api (try with sid and file name)
        if let Some(sid) = sid {
            match Beatmaps::from_osu_api(
                &sid,
                GetBeatmapMethod::Sid,
                file_name,
                &bancho.osu_api,
                database,
            )
            .await
            {
                Ok(b) => {
                    let md5 = md5.unwrap_or(&b.info.as_ref().unwrap().md5).clone();
                    cache.cache_beatmap(md5, &b).await;
                    return Some(b);
                }
                Err(err) => {
                    // If request error, we will not cache it.
                    debug!(
                        "[Beatmaps] Failed to get beatmap ({}), err: {:?};",
                        sid, err
                    );
                    if let Some(md5) = md5 {
                        if err != ApiError::RequestError {
                            // Else, cache it Not submitted
                            cache.cache_beatmap(md5.clone(), &Beatmaps::default()).await;
                        };
                    }
                }
            };
        };

        info!(
            "[Beatmaps] Failed to get beatmaps anyway, md5: {:?}, sid: {:?}.",
            md5, sid
        );
        None
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
    pub async fn from_cache(beatmap_md5: &String, cache: &Caches, expire: i64) -> Option<Self> {
        debug!("[Beatmaps] try get beatmap {} from cache...", beatmap_md5);
        let b = cache.get_beatmap(beatmap_md5).await?;
        if b.is_expired(expire) {
            debug!(
                "[Beatmaps] get from beatmap {} cache but expired, cache time: {:?}",
                beatmap_md5, b.create_time
            );
            return None;
        };
        Some(b)
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
            create_time: Local::now(),
        }
    }

    #[inline(always)]
    pub fn file_name(&self) -> String {
        utils::safe_file_name(format!(
            "{artist} - {title} ({mapper}) [{diff_name}].osu",
            artist = self.artist,
            title = self.title,
            mapper = self.mapper,
            diff_name = self.diff_name
        ))
    }

    #[inline(always)]
    pub async fn from_osu_api<T: Any + Display>(
        key: &T,
        method: GetBeatmapMethod,
        file_name: Option<&String>,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) -> Result<Self, ApiError> {
        let v = key as &dyn Any;
        let osu_api = osu_api.read().await;
        let b = match method {
            GetBeatmapMethod::Md5 => {
                osu_api
                    .fetch_beatmap_by_md5(v.downcast_ref().unwrap())
                    .await
            }
            GetBeatmapMethod::Bid => {
                osu_api
                    .fetch_beatmap_by_bid(*v.downcast_ref().unwrap())
                    .await
            }
            GetBeatmapMethod::Sid => {
                if let Some(file_name) = file_name {
                    let beatmap_list = osu_api
                        .fetch_beatmap_by_sid(*v.downcast_ref().unwrap())
                        .await?;
                    // Sid will get a list
                    let mut target = None;
                    for b in beatmap_list {
                        // Cache them
                        b.save_to_database(database).await;
                        // Try find target
                        let b_name = b.file_name();
                        let condition = &b_name == file_name;
                        debug!("[BeatmapFromApi] Check file name is correct... Current: {}, Target: {}, result: {}", b_name, file_name, condition);
                        if condition {
                            target = Some(b)
                        };
                    }
                    if let Some(b) = target {
                        debug!(
                            "[BeatmapFromApi] Success get with Method({:?}): {:?}",
                            method, b
                        );
                        return Ok(b);
                    }
                } else {
                    warn!(
                        "[BeatmapFromApi] Try get a beatmap by sid: {}, but no file_name provided.",
                        key
                    );
                }
                return Err(ApiError::NotExists);
            }
        }?;

        debug!(
            "[BeatmapFromApi] Success get with Method({:?}): {:?}",
            method, b
        );
        b.save_to_database(database).await;
        Ok(b)
    }

    #[inline(always)]
    pub async fn save_to_database(&self, database: &Database) -> bool {
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
                debug!(
                    "[BeatmapFromApi] Successfully insert beatmap({}) to database.",
                    self.md5
                );
                true
            }
            Err(err) => {
                error!(
                    "[BeatmapFromApi] Failed to insert beatmap({}) to database, err: {:?}",
                    self.md5, err
                );
                false
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
    #[inline(always)]
    pub fn get_query_fields() -> String {
        format!("\"{}\"", BeatmapInfo::FIELDS.join("\",\""))
    }

    #[inline(always)]
    pub fn file_name(&self) -> String {
        utils::safe_file_name(format!(
            "{artist} - {title} ({mapper}) [{diff_name}].osu",
            artist = self.artist,
            title = self.title,
            mapper = self.mapper,
            diff_name = self.diff_name
        ))
    }

    #[inline(always)]
    pub async fn from_database_by_bid(beatmap_id: i32, database: &Database) -> Option<Self> {
        debug!(
            "[FastDB] Try get beatmap by id: {} info from database...",
            beatmap_id
        );
        utils::struct_from_database(
            "beatmaps",
            "maps",
            "id",
            Self::get_query_fields().as_str(),
            &beatmap_id,
            database,
        )
        .await
    }
    #[inline(always)]
    pub async fn from_database_by_sid(beatmap_set_id: i32, database: &Database) -> Option<Self> {
        debug!(
            "[FastDB] Try get beatmap by set id: {} info from database...",
            beatmap_set_id
        );
        utils::struct_from_database(
            "beatmaps",
            "maps",
            "set_id",
            Self::get_query_fields().as_str(),
            &beatmap_set_id,
            database,
        )
        .await
    }
    #[inline(always)]
    pub async fn from_database_by_md5(beatmap_md5: &String, database: &Database) -> Option<Self> {
        debug!(
            "[FastDB] Try get beatmap by md5: {} info from database...",
            beatmap_md5
        );
        utils::struct_from_database(
            "beatmaps",
            "maps",
            "md5",
            Self::get_query_fields().as_str(),
            beatmap_md5,
            database,
        )
        .await
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
