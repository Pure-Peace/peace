use super::{
    caches::{Caches, PPbeatmapCache},
    glob::Glob,
};

use {
    bytes::Bytes,
    ntex::web::types::Data,
    serde::Deserialize,
    serde_json::{json, Value},
    std::{cmp::PartialEq, time::Instant},
    tokio::fs::File,
};

use peace_objects::beatmaps::traits::{BeatmapCacheStorage, MyBeatmapCache};
use peace_performance::{AnyPP, Beatmap as PPbeatmap, FruitsPP, ManiaPP, OsuPP, PpResult, TaikoPP};

macro_rules! set_calculator {
    ($target:ident.$attr:ident, $calculator:ident) => {
        match $target.$attr {
            Some($attr) => $calculator.$attr($attr),
            None => $calculator,
        }
    };
    ($target:ident.$attr:ident, $func:ident, $calculator:ident) => {
        match $target.$attr {
            Some($attr) => $calculator.$func($attr),
            None => $calculator,
        }
    };
}

#[derive(PartialEq)]
pub enum GetBeatmapError {
    FileNotFound,
    ParseError,
}

impl GetBeatmapError {
    #[inline(always)]
    pub fn error_message(&self) -> &'static str {
        match self {
            Self::FileNotFound => "cannot find .osu file",
            Self::ParseError => "cannot parse .osu file",
        }
    }

    #[inline(always)]
    pub fn error_status(&self) -> i32 {
        match self {
            Self::FileNotFound => -1,
            Self::ParseError => -2,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalcData {
    pub md5: Option<String>,
    pub bid: Option<i32>,
    pub sid: Option<i32>,
    pub file_name: Option<String>,
    pub mode: Option<u8>,
    pub mods: Option<u32>,
    pub n50: Option<usize>,
    pub n100: Option<usize>,
    pub n300: Option<usize>,
    pub katu: Option<usize>,
    pub acc: Option<f32>,
    pub passed_obj: Option<usize>,
    pub combo: Option<usize>,
    pub miss: Option<usize>,
    pub score: Option<u32>,
    pub simple: Option<i32>,
    pub acc_list: Option<i32>,
    pub no_miss: Option<i32>,
}

#[inline(always)]
pub async fn calculate_pp(beatmap: &PPbeatmap, data: &CalcData) -> PpResult {
    // Get target mode calculator
    let c = mode_calculator(data.mode.unwrap_or(4), &beatmap);
    let c = set_calculator!(data.mods, c);
    // Irrelevant for osu!mania
    let c = set_calculator!(data.combo, c);
    // Irrelevant for osu!mania and osu!taiko
    let c = set_calculator!(data.n50, c);
    // Irrelevant for osu!mania
    let c = set_calculator!(data.n100, c);
    // Irrelevant for osu!mania
    let c = set_calculator!(data.n300, c);
    // Only relevant for osu!ctb
    let c = set_calculator!(data.katu, n_katu, c);
    // Irrelevant for osu!mania
    let c = set_calculator!(data.miss, misses, c);
    let c = set_calculator!(data.passed_obj, passed_objects, c);
    // Only relevant for osu!mania
    let mut c = set_calculator!(data.score, c);
    // Irrelevant for osu!mania
    if let Some(acc) = data.acc {
        c.set_accuracy(acc)
    };

    // Calculate pp
    c.calculate().await
}

#[inline(always)]
pub async fn calculate_acc_list(beatmap: &PPbeatmap, data: &CalcData) -> Value {
    let c = mode_calculator(data.mode.unwrap_or(4), &beatmap);
    let mut c = match data.mods {
        Some(mods) => c.mods(mods),
        None => c,
    };

    let acc_100 = {
        c.set_accuracy(100.0);
        c.calculate().await
    };
    let acc_99 = {
        c.set_accuracy(99.0);
        c.calculate().await
    };
    let acc_98 = {
        c.set_accuracy(98.0);
        c.calculate().await
    };
    let acc_95 = {
        c.set_accuracy(95.0);
        c.calculate().await
    };

    json!({
        "95": acc_95.pp(),
        "98": acc_98.pp(),
        "99": acc_99.pp(),
        "100": acc_100.pp(),
    })
}

#[inline(always)]
pub fn mode_calculator(mode: u8, beatmap: &PPbeatmap) -> AnyPP {
    match mode {
        0 => AnyPP::Osu(OsuPP::new(beatmap)),
        1 => AnyPP::Taiko(TaikoPP::new(beatmap)),
        2 => AnyPP::Fruits(FruitsPP::new(beatmap)),
        3 => AnyPP::Mania(ManiaPP::new(beatmap)),
        _ => AnyPP::new(beatmap),
    }
}

#[inline(always)]
pub async fn get_beatmap(
    mut md5: Option<String>,
    mut bid: Option<i32>,
    sid: Option<i32>,
    file_name: Option<String>,
    glob: &Glob,
) -> Option<Data<PPbeatmap>> {
    let b = glob
        .caches
        .beatmap_cache
        .get(md5.as_ref(), bid, sid, file_name.as_ref())
        .await;
    if let Some(b) = b {
        #[cfg(feature = "with_peace")]
        let expire = glob.config.read().await.data.beatmaps.cache_expires;
        #[cfg(not(feature = "with_peace"))]
        let expire = glob.local_config.data.beatmap_cache_timeout as i64;

        if !b.is_expired(expire) {
            if let Some(b) = &b.beatmap {
                md5 = Some(b.md5.clone());
                bid = Some(b.id);
            }
        }
    };

    if let Ok(b) = get_beatmap_from_local(
        md5.as_ref(),
        bid,
        &glob.local_config.data.osu_files_dir,
        &glob.caches,
    )
    .await
    {
        return Some(b);
    };
    get_beatmap_from_api(md5.as_ref(), bid, sid, file_name.as_ref(), glob).await
}

#[inline(always)]
pub async fn get_beatmap_from_local(
    md5: Option<&String>,
    bid: Option<i32>,
    dir: &String,
    caches: &Data<Caches>,
) -> Result<Data<PPbeatmap>, GetBeatmapError> {
    // Try get from beatmap cache
    if let Some(md5) = md5 {
        if let Some(c) = caches.pp_beatmap_cache.read().await.get(md5) {
            let b = c.get();
            debug!("[calculate_pp] Get beatmap {}({:?}) from cache.", md5, bid);
            return Ok(b);
        };

        // Try read .osu file
        let file = match File::open(format!("{}/{}.osu", dir, md5)).await {
            Ok(file) => file,
            Err(_) => {
                info!("[calculate_pp] Cannot find .osu file, md5: '{}'", md5);
                return Err(GetBeatmapError::FileNotFound);
            }
        };

        // Try parse .osu file
        match PPbeatmap::parse(file).await {
            Ok(b) => {
                let c = PPbeatmapCache::new(b);
                let b = c.get();
                caches.cache_pp_beatmap(md5.to_string(), c).await;
                return Ok(b);
            }
            Err(err) => {
                error!(
                    "[calculate_pp] Cannot parse beatmap file, md5: '{}', err: {:?}",
                    md5, err
                );
                return Err(GetBeatmapError::ParseError);
            }
        };
    };

    if let Some(bid) = bid {
        if let Some(c) = caches
            .pp_beatmap_cache
            .read()
            .await
            .get(&format!("bid_{}", bid))
        {
            let b = c.get();
            debug!("[calculate_pp] Get beatmap {:?}({}) from cache.", md5, bid);
            return Ok(b);
        };
    };

    Err(GetBeatmapError::FileNotFound)
}

#[inline(always)]
pub async fn get_beatmap_from_api(
    request_md5: Option<&String>,
    bid: Option<i32>,
    sid: Option<i32>,
    file_name: Option<&String>,
    glob: &Glob,
) -> Option<Data<PPbeatmap>> {
    let start = Instant::now();
    let bid = if bid.is_none() {
        #[cfg(feature = "with_peace")]
        let expires = glob.config.read().await.data.beatmaps.cache_expires;
        #[cfg(feature = "with_peace")]
        let osu_api = glob.osu_api.read().await;

        #[cfg(not(feature = "with_peace"))]
        let expires = glob.local_config.data.beatmap_cache_timeout as i64;
        #[cfg(not(feature = "with_peace"))]
        let osu_api = &glob.osu_api;
        peace_objects::beatmaps::Beatmap::get(
            request_md5,
            None,
            sid,
            file_name,
            &osu_api,
            #[cfg(feature = "with_peace")]
            glob.database.get_ref(),
            true,
            &glob.caches.beatmap_cache,
            expires,
        )
        .await?
        .id
    } else {
        bid.unwrap()
    };
    // Download beatmap, and try parse it
    #[cfg(feature = "with_peace")]
    let osu_api = glob.osu_api.read().await;
    #[cfg(not(feature = "with_peace"))]
    let osu_api = &glob.osu_api;

    let (b, new_md5, bytes) = match osu_api.get_pp_beatmap(bid).await {
        Ok((b, new_md5, bytes)) => (b, new_md5, bytes),
        Err(err) => {
            warn!(
                "[calculate_pp] Cannot get .osu file from osu!api, err: {:?}",
                err
            );
            return None;
        }
    };

    // Save .osu file locally
    write_osu_file(
        bytes,
        format!("{}/{}.osu", glob.local_config.data.osu_files_dir, new_md5),
    )
    .await;

    // Cache it
    let c = PPbeatmapCache::new(b);
    let b = c.get();
    glob.caches
        .cache_pp_beatmap(new_md5.clone(), c.clone())
        .await;
    glob.caches
        .cache_pp_beatmap(format!("bid_{}", bid), c)
        .await;

    // Check .osu file is same md5
    if request_md5.is_some() && request_md5.unwrap() != &new_md5 {
        warn!("[calculate_pp] Success get .osu file from api, but md5 not eq.");
        return None;
    }

    info!(
        "[calculate_pp] Success get .osu file from api, bid: {:?}, md5: {:?}; time spent: {:?}",
        bid,
        request_md5,
        start.elapsed()
    );

    Some(b)
}

#[inline(always)]
pub async fn write_osu_file(bytes: Bytes, path: String) -> bool {
    match tokio::fs::write(path, bytes).await {
        Ok(_) => true,
        Err(err) => {
            warn!(
                "[calculate_pp] Failed to write into .osu file locally, err: {:?}",
                err
            );
            return false;
        }
    }
}
