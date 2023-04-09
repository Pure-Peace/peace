use super::{BanchoBackgroundService, PasswordCacheStore};
use crate::bancho::DynBanchoBackgroundService;
use async_trait::async_trait;
use clap_serde_derive::ClapSerde;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tools::{
    async_collections::{
        BackgroundTask, BackgroundTaskError, BackgroundTaskFactory,
        BackgroundTaskManager, CommonRecycleBackgroundTaskConfig, SignalHandle,
    },
    atomic::{Atomic, AtomicValue, U64},
    lazy_init, Timestamp,
};

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

    pub fn password_caches_recycle_factory(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    ) -> BackgroundTaskFactory {
        const LOG_TARGET: &str =
            "bancho::background_tasks::password_caches_recycling";

        let password_cache_store = self.password_cache_store.to_owned();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let password_cache_store = password_cache_store.to_owned();
            let cfg = config.to_owned();

            let task = async move {
                loop {
                    tokio::time::sleep(*cfg.loop_interval.load().as_ref())
                        .await;
                    info!(target: LOG_TARGET, "Task started!");
                    let start = Instant::now();

                    let current_timestamp = Timestamp::now();
                    let mut deactive_caches = None::<Vec<String>>;

                    let password_cache_store =
                        password_cache_store.lock().await;

                    for (key, cache) in password_cache_store.iter() {
                        if current_timestamp - cache.last_hit()
                            > cfg.dead.val() as i64
                        {
                            lazy_init!(deactive_caches => deactive_caches.push(key.to_owned()), vec![key.to_owned()]);
                        }
                    }

                    if let Some(deactive_caches) = deactive_caches {
                        let mut password_cache_store = password_cache_store;

                        for key in deactive_caches.iter() {
                            password_cache_store.remove(key);
                        }

                        info!(
                            target: LOG_TARGET,
                            "Done in: {:?} ({} caches cleared)",
                            start.elapsed(),
                            deactive_caches.len()
                        );
                    }

                    info!(
                        target: LOG_TARGET,
                        "Done in: {:?} (0 caches cleared)",
                        start.elapsed(),
                    );
                }
            };

            info!(
                target: LOG_TARGET,
                "Service started! (deactive={}s, sleep={:?})",
                config.dead.val(),
                config.loop_interval.val()
            );

            Box::pin(async move {
                tokio::select!(
                    _ = task => {},
                    _ = stop.wait_signal() => {}
                );
                warn!(target: LOG_TARGET, "Service stopped!");
            })
        }))
    }
}

#[derive(Debug, Clone, Parser, ClapSerde, Serialize, Deserialize)]
pub struct CliBanchoBackgroundServiceConfigs {
    #[default(86400)]
    #[arg(long, default_value = "86400")]
    pub password_caches_recycle_deactive_secs: u64,

    #[default(43200)]
    #[arg(long, default_value = "43200")]
    pub password_caches_recycle_interval_secs: u64,
}

pub struct PasswordCachesRecycleConfig;

impl PasswordCachesRecycleConfig {
    pub fn build(
        dead: u64,
        loop_interval: u64,
    ) -> Arc<CommonRecycleBackgroundTaskConfig> {
        CommonRecycleBackgroundTaskConfig {
            dead: U64::new(dead),
            loop_interval: Atomic::new(Duration::from_secs(loop_interval)),
            manual_stop: true.into(),
        }
        .into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct BanchoBackgroundServiceConfigs {
    pub password_caches_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
}

impl BanchoBackgroundServiceConfigs {
    pub fn new(
        password_caches_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
    ) -> Self {
        Self { password_caches_recycle }
    }
}

#[async_trait]
impl BanchoBackgroundService for BanchoBackgroundServiceImpl {
    fn start_all(&self, configs: BanchoBackgroundServiceConfigs) {
        self.start_password_caches_recycle(configs.password_caches_recycle);
    }

    fn start_password_caches_recycle(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    ) {
        self.tasks.password_caches_recycle.start(
            self.password_caches_recycle_factory(config.clone()),
            config,
        );
    }

    fn stop_password_caches_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        self.tasks.password_caches_recycle.stop()
    }
}
