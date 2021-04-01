use super::depends::*;

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
        let requester = reqwest::Client::builder()
            .connection_verbose(false)
            .timeout(Duration::from_secs(14))
            .pool_idle_timeout(Some(Duration::from_secs(300)))
            .build()
            .unwrap();
        OsuApiClient {
            key,
            requester,
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
