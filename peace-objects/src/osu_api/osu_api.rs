use peace_constants::api::ApiError;
use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "osu_file_downloader")]
use peace_performance::Beatmap as PPbeatmap;

use super::client::OsuApiClient;

const NOT_API_KEYS: &'static str = "[OsuApi] Api keys not added, could not send requests.";

#[derive(Debug)]
pub struct OsuApi {
    pub clients: Vec<OsuApiClient>,
    #[cfg(feature = "osu_file_downloader")]
    pub beatmap_downloader: OsuApiClient,
    pub delay: AtomicUsize,
    pub success_count: AtomicUsize,
    pub failed_count: AtomicUsize,
}

impl OsuApi {
    pub async fn new(api_keys: Vec<String>) -> Self {
        if api_keys.is_empty() {
            warn!("[OsuApi] No osu! apikeys has been added, please add it to the bancho.config of the database, then reload peace! Otherwise, the osu!api request cannot be used.");
        }

        let clients = api_keys
            .iter()
            .map(|key| OsuApiClient::new(key.clone()))
            .collect();

        OsuApi {
            clients,
            #[cfg(feature = "osu_file_downloader")]
            beatmap_downloader: OsuApiClient::new(String::new()),
            delay: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            failed_count: AtomicUsize::new(0),
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

    pub async fn reload_clients(&mut self, api_keys: Vec<String>) {
        let mut should_remove = vec![];

        // Remove old keys that not in new keys
        for (idx, old) in self.clients.iter().enumerate() {
            if !api_keys.contains(&old.key) {
                should_remove.push(idx);
            }
        }
        for idx in should_remove {
            let removed = self.clients.remove(idx);
            info!("[OsuApi] Removed api key and client: {}", removed.key);
        }

        // Add new clients
        let old_keys: Vec<String> = self
            .clients
            .iter()
            .map(|client| client.key.clone())
            .collect();
        for key in api_keys {
            if !old_keys.contains(&key) {
                info!("[OsuApi] Added key and client: {}", key);
                self.clients.push(OsuApiClient::new(key));
            }
        }
    }

    #[inline(always)]
    pub async fn get<T: Serialize + ?Sized>(&self, url: &str, query: &T) -> Option<Response> {
        if self.clients.is_empty() {
            error!("{}", NOT_API_KEYS);
            return None;
        }

        let mut tries = 1;
        loop {
            if tries > 3 {
                warn!("[OsuApi] Request over 3 times but still failed, stop request.");
                break;
            };
            for api_client in &self.clients {
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
                        "[OsuApi] Failed, key ({}) request-{} with: {}ms; err: {:?};",
                        api_client.key, tries, delay, res
                    );
                    api_client.failed();
                    self.failed(delay);
                    tries += 1;
                    continue;
                }
                // TODO: Statistics avg request time
                debug!(
                    "[OsuApi] Success get resp, key ({}) request-{} with: {:?}ms;",
                    api_client.key, tries, delay
                );
                api_client.success();
                self.success(delay);
                return Some(res.unwrap());
            }
        }
        None
    }

    #[inline(always)]
    pub async fn get_json<Q: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        url: &str,
        query: &Q,
    ) -> Result<T, ApiError> {
        let res = self.get(url, query).await;
        if res.is_none() {
            return Err(ApiError::RequestError);
        };
        let res = res.unwrap();
        if !res.status().is_success() {
            return Err(ApiError::RequestError);
        };
        match res.json().await {
            Ok(value) => Ok(value),
            Err(err) => {
                error!("[OsuApi] Could not parse data to json, err: {:?}", err);
                Err(ApiError::ParseError)
            }
        }
    }

    #[cfg(feature = "osu_file_downloader")]
    #[inline(always)]
    pub async fn get_pp_beatmap(
        &self,
        bid: i32,
    ) -> Result<(PPbeatmap, String, bytes::Bytes), ApiError> {
        use md5::Digest;
        let resp = self
            .beatmap_downloader
            .requester
            .get(format!("{}{}", peace_constants::api::OSU_FILE_DOWNLOAD_URL, bid).as_str())
            .send()
            .await;
        if resp.is_err() {
            return Err(ApiError::RequestError);
        };

        let bytes = resp.unwrap().bytes().await;
        if bytes.is_err() {
            return Err(ApiError::ParseError);
        };

        let bytes = bytes.unwrap();
        let b = match PPbeatmap::parse(tokio::io::Cursor::new(bytes.clone())).await {
            Ok(b) => b,
            Err(err) => {
                error!(
                    "[OsuApi] Failed to parse .osu files from requests, err: {:?}",
                    err
                );
                return Err(ApiError::ParseError);
            }
        };
        Ok((b, format!("{:x}", md5::Md5::digest(&bytes)), bytes))
    }

    pub async fn test_all(&self) -> String {
        const TEST_URL: &'static str = "https://old.ppy.sh/api/get_beatmaps";
        let mut results = json::JsonValue::new_array();

        if self.clients.is_empty() {
            error!("{}", NOT_API_KEYS);
            return NOT_API_KEYS.to_string();
        }

        for api_client in &self.clients {
            let start = std::time::Instant::now();
            let res = api_client
                .requester
                .get(TEST_URL)
                .query(&[("k", api_client.key.as_str()), ("s", "1"), ("m", "0")])
                .send()
                .await;
            let delay = start.elapsed().as_millis() as usize;

            debug!("[OsuApi] test_all request with: {:?} totaly;", delay);
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

            let _result = results.push(json::object! {
                api_key: api_client.key.clone(),
                delay: delay,
                status: status,
                error: err,
            });
        }

        results.dump()
    }
}
