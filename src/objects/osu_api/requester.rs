use super::{client::OsuApiClient, depends::*, errors::ApiError};

const NOT_API_KEYS: &'static str = "[OsuApi] Api keys not added, could not send requests.";

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
            warn!("[OsuApi] No osu! apikeys has been added, please add it to the bancho.config of the database! Otherwise, the osu!api request cannot be used.");
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
            info!("[OsuApi] Removed api key and client: {}", removed.key);
        }

        // Add new clients
        let old_keys: Vec<String> = self
            .api_clients
            .iter()
            .map(|client| client.key.clone())
            .collect();
        for key in new_keys {
            if !old_keys.contains(&key) {
                info!("[OsuApi] Added key and client: {}", key);
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

        let mut tries = 1;
        loop {
            if tries > 3 {
                warn!("[OsuApi] Request over 3 times but still failed, stop request.");
                break;
            };
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
                        "[OsuApi] key ({}) request-{} with: {}ms; err: {:?};",
                        api_client.key, tries, delay, res
                    );
                    api_client.failed();
                    self.failed(delay);
                    tries += 1;
                    continue;
                }
                info!(
                    "[OsuApi] key ({}) request-{} with: {:?}ms;",
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
            return Err(ApiError::NotExists);
        };
        match res.unwrap().json().await {
            Ok(value) => Ok(value),
            Err(err) => {
                error!("[OsuApi] Could not parse data to json, err: {:?}", err);
                Err(ApiError::ParseError)
            }
        }
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
