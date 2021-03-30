use crate::objects::osu_api::errors::ApiError;

use super::{depends::*, Beatmap, BeatmapFromApi, GetBeatmapMethod};

#[derive(Debug, Clone)]
pub struct BeatmapCache {
    pub beatmap: Option<Beatmap>,
    pub create_time: DateTime<Local>,
}

impl BeatmapCache {
    #[inline(always)]
    pub fn new(beatmap: Option<Beatmap>) -> Self {
        let create_time = match &beatmap {
            Some(b) => b.update_time,
            None => Local::now(),
        };
        BeatmapCache {
            beatmap,
            create_time,
        }
    }

    #[inline(always)]
    pub fn is_not_submit(&self) -> bool {
        self.beatmap.is_none()
    }

    #[inline(always)]
    pub async fn from_database<T: Any + Display>(
        key: &T,
        method: &GetBeatmapMethod,
        database: &Database,
    ) -> Option<Self> {
        let beatmap = Beatmap::from_database(key, &method, database).await?;
        let create_time = beatmap.update_time.clone();
        let new = Self {
            beatmap: Some(beatmap),
            create_time,
        };
        info!(
            "[BeatmapCache] get from database with Method({:?}): {}",
            method, key
        );
        Some(new)
    }

    #[inline(always)]
    pub async fn from_osu_api<T: Any + Display>(
        key: &T,
        method: &GetBeatmapMethod,
        file_name: Option<&String>,
        osu_api: &Data<RwLock<OsuApi>>,
        database: &Database,
    ) -> Result<Self, ApiError> {
        Ok(
            BeatmapFromApi::from_osu_api(key, method, file_name, osu_api, database)
                .await?
                .convert_to_beatmap_cache(),
        )
    }

    #[inline(always)]
    pub fn is_expired(&self, expires: i64) -> bool {
        if let Some(beatmap) = &self.beatmap {
            // Fixed never expire!
            if beatmap.fixed_rank_status {
                return false;
            }
        };
        (Local::now() - self.create_time).num_seconds() > expires
    }
}
