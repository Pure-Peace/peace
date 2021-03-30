use super::{depends::*, BeatmapFromApi};
use crate::utils;

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
