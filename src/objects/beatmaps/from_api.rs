use super::{depends::*, Beatmap, BeatmapCache, GetBeatmapMethod};
use crate::objects::osu_api::errors::ApiError;
use crate::utils;

#[derive(Debug, FieldNames, ToSql, Deserialize, Clone)]
pub struct BeatmapFromApi {
    #[serde(rename = "beatmap_id", with = "serde_str")]
    pub id: i32,
    #[serde(rename = "beatmapset_id", with = "serde_str")]
    pub set_id: i32,
    #[serde(rename = "file_md5")]
    pub md5: String,
    pub artist: String,
    pub artist_unicode: Option<String>,
    pub title: String,
    pub title_unicode: Option<String>,
    #[serde(rename = "creator")]
    pub mapper: String,
    #[serde(rename = "creator_id", with = "serde_str")]
    pub mapper_id: i32,
    #[serde(rename = "approved", with = "serde_str")]
    pub rank_status: i32,
    #[serde(rename = "version")]
    pub diff_name: String,
    #[serde(rename = "diff_size", with = "serde_str")]
    pub cs: f32,
    #[serde(rename = "diff_overall", with = "serde_str")]
    pub od: f32,
    #[serde(rename = "diff_approach", with = "serde_str")]
    pub ar: f32,
    #[serde(rename = "diff_drain", with = "serde_str")]
    pub hp: f32,
    #[serde(with = "serde_str")]
    pub mode: i16,
    #[serde(rename = "count_normal", with = "serde_str")]
    pub object_count: i32,
    #[serde(rename = "count_slider", with = "serde_str")]
    pub slider_count: i32,
    #[serde(rename = "count_spinner", with = "serde_str")]
    pub spinner_count: i32,
    #[serde(with = "serde_str")]
    pub bpm: f32,
    pub source: Option<String>,
    pub tags: Option<String>,
    #[serde(with = "serde_str")]
    pub genre_id: i16,
    #[serde(with = "serde_str")]
    pub language_id: i16,
    #[serde(deserialize_with = "from_str_bool")]
    pub storyboard: bool,
    #[serde(deserialize_with = "from_str_bool")]
    pub video: bool,
    #[serde(deserialize_with = "from_str_optional")]
    pub max_combo: Option<i32>,
    #[serde(rename = "total_length", with = "serde_str")]
    pub length: i32,
    #[serde(rename = "hit_length", with = "serde_str")]
    pub length_drain: i32,
    #[serde(rename = "diff_aim", deserialize_with = "from_str_optional")]
    pub aim: Option<f32>,
    #[serde(rename = "diff_speed", deserialize_with = "from_str_optional")]
    pub spd: Option<f32>,
    #[serde(rename = "difficultyrating", with = "serde_str")]
    pub stars: f32,
    #[serde(rename = "submit_date", with = "my_serde")]
    pub submit_time: Option<DateTime<Local>>,
    #[serde(rename = "approved_date", with = "my_serde")]
    pub approved_time: Option<DateTime<Local>>,
    #[serde(with = "my_serde")]
    pub last_update: Option<DateTime<Local>>,
}

impl BeatmapFromApi {
    #[inline(always)]
    pub fn convert_to_beatmap(self) -> Beatmap {
        Beatmap::from(self)
    }

    #[inline(always)]
    pub fn convert_to_beatmap_cache(self) -> BeatmapCache {
        BeatmapCache {
            beatmap: Some(Beatmap::from(self)),
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
        method: &GetBeatmapMethod,
        file_name: Option<&String>,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) -> Result<Self, ApiError> {
        let v = key as &dyn Any;
        let osu_api = osu_api.read().await;
        let b = match method {
            &GetBeatmapMethod::Md5 => {
                osu_api
                    .fetch_beatmap_by_md5(v.downcast_ref().unwrap())
                    .await
            }
            &GetBeatmapMethod::Bid => {
                osu_api
                    .fetch_beatmap_by_bid(*v.downcast_ref().unwrap())
                    .await
            }
            &GetBeatmapMethod::Sid => {
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
                    "INSERT INTO \"beatmaps\".\"maps\" (\"{}\") VALUES ({}) 
                        ON CONFLICT (\"md5\") DO UPDATE SET 
                            rank_status = EXCLUDED.rank_status,
                            approved_time = EXCLUDED.approved_time;",
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
