use async_std::sync::RwLock;
use hashbrown::HashMap;

use crate::types::{Argon2Cache, BeatmapsCache};

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
}
