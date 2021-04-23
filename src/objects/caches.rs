use std::sync::atomic::AtomicI32;

use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashMap;
use peace_objects::beatmaps::CommonBeatmapCaches;

use crate::types::{Argon2Cache, TempTableCache};

pub struct Caches {
    pub beatmap_cache: CommonBeatmapCaches,
    pub argon2_cache: RwLock<Argon2Cache>,
    pub temp_table_cache: RwLock<TempTableCache>,
}

impl Caches {
    pub fn new() -> Self {
        Caches {
            beatmap_cache: CommonBeatmapCaches {
                md5: RwLock::new(HashMap::with_capacity(500)),
                bid: RwLock::new(HashMap::with_capacity(500)),
                sid: RwLock::new(HashMap::with_capacity(500)),
                length: AtomicI32::new(0),
            },
            argon2_cache: RwLock::new(HashMap::with_capacity(1000)),
            temp_table_cache: RwLock::new(HashMap::with_capacity(1000)),
        }
    }

    #[inline(always)]
    pub async fn cache_temp_table(&self, table_name: String) -> Option<DateTime<Local>> {
        self.temp_table_cache
            .write()
            .await
            .insert(table_name, Local::now())
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

    #[inline(always)]
    pub async fn get_temp_table(&self, table_name: &String) -> Option<DateTime<Local>> {
        self.temp_table_cache.read().await.get(table_name).cloned()
    }
}
