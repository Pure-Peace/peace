use derivative::Derivative;
use postgres_types::ToSql;
use serde::{Deserialize, Serialize};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crate::settings::local::Settings;

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
    pub fn new(settings: &Settings) -> Self {
        let client = reqwest::Client::builder()
            .connection_verbose(false)
            .timeout(Duration::from_secs(settings.pp_server.pp_calc_timeout))
            .pool_idle_timeout(Some(Duration::from_secs(300)))
            .build()
            .unwrap();
        Self {
            url: settings.pp_server.url.clone(),
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

#[derive(Debug, Serialize, Deserialize, ToSql)]
pub struct PPRaw {
    pub acc: Option<f32>,
    pub aim: Option<f32>,
    pub spd: Option<f32>,
    pub str: Option<f32>,
    pub total: f32,
}

#[derive(Debug, Deserialize)]
pub struct PPAcclist {
    pub a95: f32,
    pub a98: f32,
    pub a99: f32,
    pub a100: f32,
}

#[derive(Debug, Deserialize)]
pub struct PPCalcResult {
    pub message: String,
    pub status: i32,
    pub mode: u8,
    pub mods: u32,
    pub pp: f32,
    pub stars: f32,
    pub raw: Option<PPRaw>,
    pub acc_list: Option<PPAcclist>,
}
