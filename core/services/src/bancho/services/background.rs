use super::{BanchoBackgroundService, PasswordCacheStore};
use crate::bancho::DynBanchoBackgroundService;
use async_trait::async_trait;
use chrono::Utc;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tools::async_collections::{
    BackgroundTask, BackgroundTaskError, BackgroundTaskFactory,
    BackgroundTaskManager, SignalHandle,
};

const DEACTIVE: i64 = 86400;
const SLEEP: Duration = Duration::from_secs(86400);

#[derive(Clone, Default)]
pub struct Tasks {
    password_caches_recycle: BackgroundTaskManager,
}

#[derive(Clone)]
pub struct BanchoBackgroundServiceImpl {
    password_cache_store: PasswordCacheStore,
    tasks: Tasks,
}

impl BanchoBackgroundServiceImpl {
    pub fn into_service(self) -> DynBanchoBackgroundService {
        Arc::new(self) as DynBanchoBackgroundService
    }

    pub fn new(password_cache_store: PasswordCacheStore) -> Self {
        Self { password_cache_store, tasks: Tasks::default() }
    }

    pub fn password_caches_recycle_factory(&self) -> BackgroundTaskFactory {
        const LOG_TARGET: &str =
            "bancho::background_tasks::password_caches_recycling";

        let password_cache_store = self.password_cache_store.to_owned();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let password_cache_store = password_cache_store.to_owned();
            let task = async move {
                loop {
                    tokio::time::sleep(SLEEP).await;
                    info!(
                        target: LOG_TARGET,
                        "Task started!"
                    );
                    let start = Instant::now();

                    let current_timestamp = Utc::now().timestamp();

                    let password_cache_store =
                        password_cache_store.lock().await;

                    let deactive_caches = password_cache_store
                        .iter()
                        .filter(|(_, cache)| {
                            current_timestamp - cache.last_hit().timestamp()
                                > DEACTIVE
                        })
                        .map(|(k, _)| k.to_owned())
                        .collect::<Vec<String>>();

                    let mut password_cache_store = password_cache_store;

                    for key in deactive_caches.iter() {
                        password_cache_store.remove(key);
                    }

                    info!(target: LOG_TARGET,
                        "Done in: {:?} ({} caches cleared)",
                        start.elapsed(), deactive_caches.len()
                    );
                }
            };

            Box::pin(async move {
                info!(
                    target: LOG_TARGET,
                    "Service started! (deactive={}s, sleep={:?})",
                    DEACTIVE,
                    SLEEP
                );
                tokio::select!(
                    _ = task => {},
                    _ = stop.wait_signal() => {}
                );
                warn!(target: LOG_TARGET, "Service stopped!");
            })
        }))
    }
}

#[async_trait]
impl BanchoBackgroundService for BanchoBackgroundServiceImpl {
    fn start_all(&self) {
        self.start_password_caches_recycle();
    }

    fn start_password_caches_recycle(&self) {
        self.tasks
            .password_caches_recycle
            .start(self.password_caches_recycle_factory(), true);
    }

    fn stop_password_caches_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        self.tasks.password_caches_recycle.stop()
    }
}
