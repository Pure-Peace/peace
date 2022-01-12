use {
    chrono::{DateTime, Local},
    hashbrown::HashMap,
    ntex::web::types::Data,
    std::sync::atomic::AtomicI32,
    tokio::sync::RwLock,
};

use peace_objects::beatmaps::CommonBeatmapCaches;
use peace_performance::Beatmap as PPbeatmap;

use crate::settings::model::LocalConfigData;

#[derive(Clone)]
pub struct PPbeatmapCache {
    pub beatmap: Data<PPbeatmap>,
    pub time: DateTime<Local>,
}

impl PPbeatmapCache {
    #[inline(always)]
    pub fn new(beatmap: PPbeatmap) -> Self {
        Self {
            beatmap: Data::new(beatmap),
            time: Local::now(),
        }
    }

    #[inline(always)]
    pub fn get(&self) -> Data<PPbeatmap> {
        self.beatmap.clone()
    }
}

pub struct Caches {
    pub beatmap_cache: CommonBeatmapCaches,
    pub pp_beatmap_cache: RwLock<HashMap<String, PPbeatmapCache>>,
    pub config: LocalConfigData,
}

impl Caches {
    pub fn new(config: LocalConfigData) -> Self {
        Self {
            beatmap_cache: CommonBeatmapCaches {
                md5: RwLock::new(HashMap::with_capacity(500)),
                bid: RwLock::new(HashMap::with_capacity(500)),
                sid: RwLock::new(HashMap::with_capacity(500)),
                length: AtomicI32::new(0),
            },
            pp_beatmap_cache: RwLock::new(HashMap::with_capacity(200)),
            config,
        }
    }

    #[inline(always)]
    pub async fn cache_pp_beatmap(&self, md5: String, pp_beatmap_cache: PPbeatmapCache) {
        let mut cw = self.pp_beatmap_cache.write().await;
        if cw.len() as i32 > self.config.beatmap_cache_max {
            debug!("[pp_beatmap_cache] Cache exceed max limit.");
            return;
        };
        cw.insert(md5, pp_beatmap_cache);
    }
}
