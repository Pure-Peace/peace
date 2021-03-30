use crate::objects::osu_api::errors::ApiError;

use super::{depends::*, BeatmapFromApi, BeatmapInfo, GetBeatmapMethod};

#[derive(Debug, Clone)]
pub struct Beatmap {
    pub info: Option<BeatmapInfo>,
    pub create_time: DateTime<Local>,
}

impl Beatmap {
    #[inline(always)]
    pub fn default() -> Self {
        Beatmap {
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
            "[Beatmap] get from database with Method({:?}): {}",
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
                    info!("[Beatmap] Get from cache: {};", md5);
                    return Some(b);
                };
            };

            // Local cache expired or not founded, then
            // Try get beatmap from database
            if let Some(b) = Beatmap::from_database(md5, GetBeatmapMethod::Md5, database).await {
                // If not expired, cache it locally and returns it.
                if !b.is_expired(expire) {
                    cache.cache_beatmap(md5.clone(), &b).await;
                    return Some(b);
                }
            };

            // Database cache expired or not founded, then
            // Try get beatmap from osu!api (try with md5)
            match Beatmap::from_osu_api(md5, GetBeatmapMethod::Md5, None, &bancho.osu_api, database)
                .await
            {
                Ok(b) => {
                    cache.cache_beatmap(md5.clone(), &b).await;
                    return Some(b);
                }
                Err(err) => {
                    // If request error, we will not cache it.
                    debug!("[Beatmap] Failed to get beatmap ({}), err: {:?};", md5, err);
                    if err != ApiError::RequestError {
                        // Else, cache it Not submitted
                        cache.cache_beatmap(md5.clone(), &Beatmap::default()).await;
                    };
                }
            };
        };

        // Cannot get from osu!api from md5, then
        // If SID Available,
        // Try get beatmap from osu!api (try with sid and file name)
        if let Some(sid) = sid {
            match Beatmap::from_osu_api(
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
                    debug!("[Beatmap] Failed to get beatmap ({}), err: {:?};", sid, err);
                    if let Some(md5) = md5 {
                        if err != ApiError::RequestError {
                            // Else, cache it Not submitted
                            cache.cache_beatmap(md5.clone(), &Beatmap::default()).await;
                        };
                    }
                }
            };
        };

        info!(
            "[Beatmap] Failed to get beatmaps anyway, md5: {:?}, sid: {:?}.",
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
        debug!("[Beatmap] try get beatmap {} from cache...", beatmap_md5);
        let b = cache.get_beatmap(beatmap_md5).await?;
        if b.is_expired(expire) {
            debug!(
                "[Beatmap] get from beatmap {} cache but expired, cache time: {:?}",
                beatmap_md5, b.create_time
            );
            return None;
        };
        Some(b)
    }
}
