#[cfg(feature = "async_trait")]
use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};

#[cfg(feature = "async_trait")]
use async_trait::async_trait;
use peace_constants::api::{ApiError, GetBeatmapMethod};
#[cfg(feature = "with_peace")]
use peace_database::Database;

use super::traits::{BeatmapCacheStorage, MyBeatmapCache};
use super::BeatmapFromApi;

use crate::beatmaps::Beatmap;
use crate::osu_api::OsuApi;

#[derive(Debug, Clone)]
pub struct GenericBeatmapCache<T> {
    pub beatmap: Option<T>,
    pub create_time: DateTime<Local>,
}

pub type BeatmapCache = GenericBeatmapCache<Beatmap>;

impl MyBeatmapCache<Beatmap> for BeatmapCache {
    #[inline(always)]
    fn new(beatmap: Option<Beatmap>) -> Self {
        Self {
            beatmap,
            create_time: Local::now(),
        }
    }

    #[inline(always)]
    fn is_expired(&self, expires: i64) -> bool {
        if let Some(beatmap) = &self.beatmap {
            // Fixed never expire!
            if beatmap.fixed_rank_status {
                return false;
            }
        };
        (Local::now() - self.create_time).num_seconds() > expires
    }
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

    #[cfg(feature = "with_peace")]
    #[inline(always)]
    pub async fn from_database(method: &GetBeatmapMethod, database: &Database) -> Option<Self> {
        let beatmap = Beatmap::from_database(method, database).await?;
        let create_time = beatmap.update_time.clone();
        let new = Self {
            beatmap: Some(beatmap),
            create_time,
        };
        info!("[BeatmapCache] get from database with {:?}", method);
        Some(new)
    }

    #[inline(always)]
    pub async fn from_osu_api(
        method: &GetBeatmapMethod,
        file_name: Option<&String>,
        osu_api: &OsuApi,
        #[cfg(feature = "with_peace")] database: &Database,
    ) -> Result<Self, ApiError> {
        Ok(BeatmapFromApi::from_osu_api(
            method,
            file_name,
            osu_api,
            #[cfg(feature = "with_peace")]
            database,
        )
        .await?
        .convert_to_beatmap_cache())
    }
}

pub type CommonBeatmapCaches = GenericBeatmapCaches<BeatmapCache>;

pub struct GenericBeatmapCaches<T> {
    #[cfg(feature = "async_trait")]
    pub md5: RwLock<HashMap<String, T>>,
    #[cfg(feature = "async_trait")]
    pub bid: RwLock<HashMap<i32, T>>,
    #[cfg(feature = "async_trait")]
    pub sid: RwLock<HashMap<String, T>>,
    pub length: AtomicI32,
}

#[cfg(feature = "async_trait")]
#[async_trait]
impl<
        B: Sync + std::marker::Send + Clone,
        T: Sync + std::marker::Send + Clone + MyBeatmapCache<B>,
    > BeatmapCacheStorage<B, T> for GenericBeatmapCaches<T>
{
    #[inline(always)]
    async fn get(
        &self,
        md5: Option<&String>,
        bid: Option<i32>,
        sid: Option<i32>,
        file_name: Option<&String>,
    ) -> Option<T> {
        if let Some(md5) = md5 {
            if let Some(b) = self.md5.read().await.get(md5).cloned() {
                return Some(b);
            }
        };
        if let Some(bid) = &bid {
            if let Some(b) = self.bid.read().await.get(bid).cloned() {
                return Some(b);
            }
        };
        if let (Some(sid), Some(f)) = (&sid, file_name) {
            if let Some(b) = self
                .sid
                .read()
                .await
                .get(&format!("{}_{}", sid, f))
                .cloned()
            {
                return Some(b);
            }
        };
        None
    }

    #[inline(always)]
    async fn cache(
        &self,
        md5: Option<&String>,
        bid: Option<i32>,
        sid: Option<i32>,
        file_name: Option<&String>,
        beatmap: Option<&B>,
    ) -> Option<T> {
        let mut result = None;
        if let Some(md5) = md5 {
            result = self
                .md5
                .write()
                .await
                .insert(md5.clone(), T::new(beatmap.cloned()));
        };
        if let Some(bid) = bid {
            result = self.bid.write().await.insert(bid, T::new(beatmap.cloned()));
        };
        if let (Some(sid), Some(f)) = (sid, file_name) {
            result = self
                .sid
                .write()
                .await
                .insert(format!("{}_{}", sid, f), T::new(beatmap.cloned()));
        };
        if result.is_some() {
            self.length.fetch_add(1, Ordering::SeqCst);
        };
        result
    }

    #[inline(always)]
    async fn cache_with_md5(&self, md5: &String, beatmap: Option<&B>) -> Option<T> {
        self.cache(Some(md5), None, None, None, beatmap).await
    }

    #[inline(always)]
    async fn cache_with_bid(&self, bid: i32, beatmap: Option<&B>) -> Option<T> {
        self.cache(None, Some(bid), None, None, beatmap).await
    }

    #[inline(always)]
    async fn cache_with_sid(&self, sid: i32, file_name: &String, beatmap: Option<&B>) -> Option<T> {
        self.cache(None, None, Some(sid), Some(file_name), beatmap)
            .await
    }

    #[inline(always)]
    async fn clean(&self) -> i32 {
        let (mut md5, mut bid, mut sid) = (
            self.md5.write().await,
            self.bid.write().await,
            self.sid.write().await,
        );
        (md5.clear(), bid.clear(), sid.clear());
        self.length.swap(0, Ordering::SeqCst)
    }

    #[inline(always)]
    async fn remove_timeouted(&self, expires: i64) -> i32 {
        let (mut md5, mut bid, mut sid) = (Vec::new(), Vec::new(), Vec::new());
        {
            {
                let lock = self.md5.read().await;
                for (key, b) in lock.iter() {
                    if b.is_expired(expires) {
                        md5.push(key.clone())
                    };
                }
            }
            {
                let mut lock = self.md5.write().await;
                for key in &md5 {
                    lock.remove(key);
                }
            }
        }
        {
            {
                let lock = self.bid.read().await;
                for (key, b) in lock.iter() {
                    if b.is_expired(expires) {
                        bid.push(key.clone())
                    };
                }
            }
            {
                let mut lock = self.bid.write().await;
                for key in &bid {
                    lock.remove(key);
                }
            }
        }
        {
            {
                let lock = self.sid.read().await;
                for (key, b) in lock.iter() {
                    if b.is_expired(expires) {
                        sid.push(key.clone())
                    };
                }
            }
            {
                let mut lock = self.sid.write().await;
                for key in &sid {
                    lock.remove(key);
                }
            }
        }
        self.length.swap(0, Ordering::SeqCst)
    }
}
