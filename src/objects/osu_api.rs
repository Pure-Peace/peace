use async_std::sync::RwLock;
use derivative::Derivative;
use json::object;
use reqwest::{Error, Response};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::settings::bancho::BanchoConfig;

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
        let api_clients = bancho_config
            .read()
            .await
            .osu_api_keys
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

    pub async fn get<T: Serialize + ?Sized>(
        &self,
        url: &str,
        query: &T,
    ) -> Result<Response, Error> {
        let mut err: Option<Result<Response, Error>> = None;

        for api_client in &self.api_clients {
            let start = std::time::Instant::now();
            let res = api_client.requester.get(url).query(query).send().await;
            let delay = start.elapsed().as_millis() as usize;
            if res.is_err() {
                warn!(
                    "[failed] osu! api({}) request with: {}ms; err: {:?};",
                    api_client.key, delay, res
                );
                api_client.failed();
                self.failed(delay);
                err = Some(res);
                continue;
            }

            debug!(
                "[success] osu! api({}) request with: {:?};",
                api_client.key, delay
            );
            api_client.success();
            self.success(delay);
            return res;
        }
        err.unwrap()
    }

    pub async fn test_all(&self) -> String {
        const TEST_URL: &'static str = "https://old.ppy.sh/api/get_beatmaps";
        let mut results = json::JsonValue::new_array();

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
