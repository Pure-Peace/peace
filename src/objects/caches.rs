use async_std::sync::RwLock;
use hashbrown::HashMap;

use crate::types::{Argon2Cache, BeatmapsCache};

use super::Beatmap;

pub struct Caches {
    pub beatmaps_cache: RwLock<BeatmapsCache>,
    pub argon2_cache: RwLock<Argon2Cache>,
}

impl Caches {
    pub fn new() -> Self {
        Caches {
            beatmaps_cache: RwLock::new(HashMap::with_capacity(2000)),
            argon2_cache: RwLock::new(HashMap::with_capacity(1000)),
        }
    }

    #[inline(always)]
    pub async fn cache_beatmap(&self, beatmap_md5: String, beatmaps: &Beatmap) -> Option<Beatmap> {
        self.beatmaps_cache
            .write()
            .await
            .insert(beatmap_md5, beatmaps.clone())
    }

    #[inline(always)]
    pub async fn get_beatmap(&self, beatmap_md5: &String) -> Option<Beatmap> {
        self.beatmaps_cache.read().await.get(beatmap_md5).cloned()
    }

    #[inline(always)]
    pub async fn cache_password(&self, argon2: &String, password: &String) -> Option<String> {
        self.argon2_cache
            .write()
            .await
            .insert(argon2.to_string(), password.to_string())
    }

    #[inline(always)]
    pub async fn get_password(&self, argon2: &String) -> Option<String> {
        self.argon2_cache.read().await.get(argon2).cloned()
    }
}
