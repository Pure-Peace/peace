use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use derivative::Derivative;
use json::object;
use reqwest::Response;
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serde_str;
use crate::utils::from_str_bool;

use crate::settings::bancho::BanchoConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct BeatmapFromApi {
    #[serde(rename = "beatmap_id", with = "serde_str")]
    id: i32,
    #[serde(rename = "beatmapset_id", with = "serde_str")]
    set_id: i32,
    #[serde(rename = "file_md5")]
    md5: String,
    artist: String,
    artist_unicode: Option<String>,
    title: String,
    title_unicode: Option<String>,
    #[serde(rename = "creator")]
    mapper: String,
    #[serde(rename = "creator_id", with = "serde_str")]
    mapper_id: i32,
    #[serde(rename = "approved", with = "serde_str")]
    rank_status: i32,
    #[serde(rename = "version")]
    diff_name: String,
    #[serde(rename = "diff_size", with = "serde_str")]
    cs: f32,
    #[serde(rename = "diff_overall", with = "serde_str")]
    od: f32,
    #[serde(rename = "diff_approach", with = "serde_str")]
    ar: f32,
    #[serde(rename = "diff_drain", with = "serde_str")]
    hp: f32,
    #[serde(with = "serde_str")]
    mode: i16,
    #[serde(rename = "count_normal", with = "serde_str")]
    object_count: i32,
    #[serde(rename = "count_slider", with = "serde_str")]
    slider_count: i32,
    #[serde(rename = "count_spinner", with = "serde_str")]
    spinner_count: i32,
    #[serde(with = "serde_str")]
    bpm: f32,
    source: Option<String>,
    tags: Option<String>,
    #[serde(with = "serde_str")]
    genre_id: i16,
    #[serde(with = "serde_str")]
    language_id: i16,
    #[serde(deserialize_with  = "from_str_bool")]
    storyboard: bool,
    #[serde(deserialize_with  = "from_str_bool")]
    video: bool,
    #[serde(with = "serde_str")]
    max_combo: i32,
    #[serde(rename = "total_length", with = "serde_str")]
    length: i32,
    #[serde(rename = "hit_length", with = "serde_str")]
    length_drain: i32,
    #[serde(rename = "diff_aim", with = "serde_str")]
    aim: f32,
    #[serde(rename = "diff_speed", with = "serde_str")]
    spd: f32,
    #[serde(rename = "difficultyrating", with = "serde_str")]
    stars: f32,
    #[serde(rename = "submit_date", with = "my_serde")]
    submit_time: Option<DateTime<Local>>,
    #[serde(rename = "approved_date", with = "my_serde")]
    approved_time: Option<DateTime<Local>>,
    #[serde(with = "my_serde")]
    last_update: Option<DateTime<Local>>,
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
        let s = String::deserialize(deserializer)?;
        Local
            .datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
            .map(|f| Some(f))
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OsuApiClient {
    pub key: String,
    #[derivative(Debug = "ignore")]
    pub requester: reqwest::Client,
    pub success_count: AtomicUsize,
    pub failed_count: AtomicUsize,
}

impl OsuApiClient {
    pub fn new(key: String) -> Self {
        OsuApiClient {
            key,
            requester: reqwest::Client::new(),
            success_count: AtomicUsize::new(0),
            failed_count: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn success(&self) {
        self.success_count.fetch_add(1, Ordering::SeqCst);
    }

    #[inline(always)]
    pub fn failed(&self) {
        self.failed_count.fetch_add(1, Ordering::SeqCst);
    }
}

const NOT_API_KEYS: &'static str = "osu! apikeys not added, could not send requests.";

#[derive(Derivative)]
#[derivative(Debug)]
pub struct OsuApi {
    pub api_clients: Vec<OsuApiClient>,
    pub delay: AtomicUsize,
    pub success_count: AtomicUsize,
    pub failed_count: AtomicUsize,
    #[derivative(Debug = "ignore")]
    _bancho_config: Arc<RwLock<BanchoConfig>>,
}

impl OsuApi {
    pub async fn new(bancho_config: &Arc<RwLock<BanchoConfig>>) -> Self {
        let api_keys = &bancho_config.read().await.osu_api_keys;

        if api_keys.is_empty() {
            warn!("osu! api: No osu! apikeys has been added, please add it to the bancho.config of the database! Otherwise, the osu!api request cannot be used.");
        }

        let api_clients = api_keys
            .iter()
            .map(|key| OsuApiClient::new(key.clone()))
            .collect();

        OsuApi {
            api_clients,
            delay: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            failed_count: AtomicUsize::new(0),
            _bancho_config: bancho_config.clone(),
        }
    }

    #[inline(always)]
    pub fn success(&self, delay: usize) {
        self.success_count.fetch_add(1, Ordering::SeqCst);
        self.delay.swap(delay, Ordering::SeqCst);
    }

    #[inline(always)]
    pub fn failed(&self, delay: usize) {
        self.failed_count.fetch_add(1, Ordering::SeqCst);
        self.delay.swap(delay, Ordering::SeqCst);
    }

    pub async fn reload_clients(&mut self) {
        let new_keys = self._bancho_config.read().await.osu_api_keys.clone();

        // Remove client if not exists in new keys
        let mut should_remove = vec![];
        for (idx, client) in self.api_clients.iter().enumerate() {
            if !new_keys.contains(&client.key) {
                should_remove.push(idx);
            }
        }
        for idx in should_remove {
            let removed = self.api_clients.remove(idx);
            info!("osu api: Removed key {}", removed.key);
        }

        // Add new clients
        let old_keys: Vec<String> = self
            .api_clients
            .iter()
            .map(|client| client.key.clone())
            .collect();
        for key in new_keys {
            if !old_keys.contains(&key) {
                info!("osu api: Added key {}", key);
                self.api_clients.push(OsuApiClient::new(key));
            }
        }
    }

    #[inline(always)]
    pub async fn get<T: Serialize + ?Sized>(&self, url: &str, query: &T) -> Option<Response> {
        if self.api_clients.is_empty() {
            error!("{}", NOT_API_KEYS);
            return None;
        }

        for api_client in &self.api_clients {
            let start = std::time::Instant::now();
            let res = api_client
                .requester
                .get(url)
                .query(&[("k", api_client.key.as_str())])
                .query(query)
                .send()
                .await;
            let delay = start.elapsed().as_millis() as usize;
            if res.is_err() {
                warn!(
                    "[failed] osu! api({}) request with: {}ms; err: {:?};",
                    api_client.key, delay, res
                );
                api_client.failed();
                self.failed(delay);
                continue;
            }

            debug!(
                "[success] osu! api({}) request with: {:?}ms;",
                api_client.key, delay
            );
            api_client.success();
            self.success(delay);
            return Some(res.unwrap());
        }
        None
    }

    #[inline(always)]
    pub async fn get_json<Q: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        url: &str,
        query: &Q,
    ) -> Option<T> {
        let res = self.get(url, query).await?;
        match res.json().await {
            Ok(value) => Some(value),
            Err(err) => {
                error!(
                    "[failed] osu! api could not parse data to json, err: {:?}",
                    err
                );
                None
            }
        }
    }

    #[inline(always)]
    pub async fn fetch_beatmap(&self, beatmap_hash: &String) -> Option<Vec<BeatmapFromApi>> {
        self.get_json(
            "https://old.ppy.sh/api/get_beatmaps",
            &[("h", beatmap_hash)],
        )
        .await?
    }

    pub async fn test_all(&self) -> String {
        const TEST_URL: &'static str = "https://old.ppy.sh/api/get_beatmaps";
        let mut results = json::JsonValue::new_array();

        if self.api_clients.is_empty() {
            error!("{}", NOT_API_KEYS);
            return NOT_API_KEYS.to_string();
        }

        for api_client in &self.api_clients {
            let start = std::time::Instant::now();
            let res = api_client
                .requester
                .get(TEST_URL)
                .query(&[("k", api_client.key.as_str()), ("s", "1"), ("m", "0")])
                .send()
                .await;
            let delay = start.elapsed().as_millis() as usize;

            debug!("osu! api test request with: {:?};", delay);
            let (status, err) = match res {
                Ok(resp) => {
                    api_client.success();
                    self.success(delay);
                    (resp.status() == 200, "".to_string())
                }
                Err(err) => {
                    api_client.failed();
                    self.failed(delay);
                    (false, err.to_string())
                }
            };

            let _result = results.push(object! {
                api_key: api_client.key.clone(),
                delay: delay,
                status: status,
                error: err,
            });
        }

        results.dump()
    }
}
