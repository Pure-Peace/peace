use peace_constants::PPCalcResult;
use derivative::Derivative;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PPServerApi {
    pub url: String,
    #[derivative(Debug = "ignore")]
    pub client: reqwest::Client,
    pub delay: AtomicUsize,
    pub success_count: AtomicUsize,
    pub failed_count: AtomicUsize,
}

impl PPServerApi {
    #[inline(always)]
    pub fn new(url: String, pp_calc_timeout: u64) -> Self {
        let client = reqwest::Client::builder()
            .connection_verbose(false)
            .timeout(Duration::from_secs(pp_calc_timeout))
            .pool_idle_timeout(Some(Duration::from_secs(300)))
            .build()
            .unwrap();
        Self {
            url,
            client,
            delay: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            failed_count: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub async fn calc(&self, query: &str) -> Option<PPCalcResult> {
        let start = std::time::Instant::now();
        let res = self
            .client
            .get(&format!("{}/api/calc?{}", self.url, query))
            .send()
            .await;
        let delay = start.elapsed().as_millis() as usize;
        debug!("[PPServerApi] Request with: {}ms;", delay);
        if res.is_err() {
            self.failed(delay);
            warn!("[PPServerApi] Request failed, err: {:?};", res);
            return None;
        };
        match res.unwrap().json::<PPCalcResult>().await {
            Ok(r) => {
                if r.status == 1 {
                    self.success(delay);
                    debug!(
                        "[PPServerApi] Calc success, query: {}, data: {:?}",
                        query, r
                    );
                    Some(r)
                } else {
                    self.failed(delay);
                    warn!("[PPServerApi] Calc error, query: {}, data: {:?}", query, r);
                    None
                }
            }
            Err(err) => {
                self.failed(delay);
                error!(
                    "[PPServerApi] Parse error, query: {}, err: {:?}",
                    query, err
                );
                None
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
