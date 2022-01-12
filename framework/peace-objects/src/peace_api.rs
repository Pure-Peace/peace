use derivative::Derivative;
use peace_constants::api::ApiError;
use reqwest::{RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

const REQUEST_TIMEOUT: u64 = 15;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PeaceApi {
    pub key: String,
    pub url: String,
    #[derivative(Debug = "ignore")]
    pub client: reqwest::Client,
    pub delay: AtomicUsize,
    pub success_count: AtomicUsize,
    pub failed_count: AtomicUsize,
}

impl PeaceApi {
    #[inline(always)]
    pub fn new(key: String, url: String) -> Self {
        let client = reqwest::Client::builder()
            .connection_verbose(false)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT))
            .pool_idle_timeout(Some(Duration::from_secs(300)))
            .build()
            .unwrap();
        Self {
            key,
            url,
            client,
            delay: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            failed_count: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub async fn request_wrapper(&self, req: RequestBuilder) -> Option<Response> {
        let start = std::time::Instant::now();
        let res = req.header("peace_key", &self.key).send().await;
        let delay = start.elapsed().as_millis() as usize;
        if res.is_err() {
            warn!(
                "[PeaceApi/get] Failed, key ({}) request with: {}ms; err: {:?};",
                self.key, delay, res
            );
            self.failed(delay);
            return None;
        }
        // TODO: Statistics avg request time
        debug!(
            "[PeaceApi/get] Success get resp, key ({}) request with: {:?}ms;",
            self.key, delay
        );
        self.success(delay);
        Some(res.unwrap())
    }

    #[inline(always)]
    pub fn full_url(&self, path: &str) -> String {
        format!("{}/{}", self.url, path)
    }

    #[inline(always)]
    pub async fn post<T: Serialize + ?Sized>(&self, path: &str, json: &T) -> Option<Response> {
        self.request_wrapper(self.client.post(&self.full_url(path)).json(json)).await
    }

    #[inline(always)]
    pub async fn get<T: Serialize + ?Sized>(&self, path: &str, query: &T) -> Option<Response> {
        self.request_wrapper(self.client.get(&self.full_url(path)).query(query)).await
    }

    #[inline(always)]
    pub async fn simple_get(&self, path: &str) -> Option<Response> {
        self.request_wrapper(self.client.get(&self.full_url(path))).await
    }

    #[inline(always)]
    pub async fn get_json<Q: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T, ApiError> {
        let res = match self.get(path, query).await {
            Some(r) => {
                if !r.status().is_success() {
                    return Err(ApiError::RequestError);
                };
                r
            }
            None => {
                return Err(ApiError::RequestError);
            }
        };

        match res.json().await {
            Ok(value) => Ok(value),
            Err(err) => {
                error!("[PeaceApi] Could not parse data to json, err: {:?}", err);
                Err(ApiError::ParseError)
            }
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
}
