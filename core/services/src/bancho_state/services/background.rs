use super::BanchoStateBackgroundService;
use crate::bancho_state::{
    DynBanchoStateBackgroundService, DynUserSessionsService, Session,
};
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
    user_sessions_recycle: BackgroundTaskManager,
}

#[derive(Clone)]
pub struct BanchoStateBackgroundServiceImpl {
    user_sessions_service: DynUserSessionsService,
    tasks: Tasks,
}

impl BanchoStateBackgroundServiceImpl {
    pub fn into_service(self) -> DynBanchoStateBackgroundService {
        Arc::new(self) as DynBanchoStateBackgroundService
    }

    pub fn new(user_sessions_service: DynUserSessionsService) -> Self {
        Self { user_sessions_service, tasks: Tasks::default() }
    }

    pub fn user_sessions_recycle_factory(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    ) -> BackgroundTaskFactory {
        const LOG_TARGET: &str =
            "bancho_state::background_tasks::user_sessions_recycling";

        let user_sessions_service = self.user_sessions_service.to_owned();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let user_sessions_service = user_sessions_service.to_owned();
            let cfg = config.to_owned();

            let task = async move {
                loop {
                    tokio::time::sleep(*cfg.loop_interval.load().as_ref())
                        .await;
                    info!(target: LOG_TARGET, "Task started!");
                    let start = Instant::now();

                    let current_timestamp = Timestamp::now();
                    let mut users_deactive = None::<Vec<Arc<Session>>>;

                    {
                        let user_sessions =
                            user_sessions_service.user_sessions().read().await;

                        for session in user_sessions.values() {
                            if current_timestamp - session.last_active.val()
                                > cfg.dead.val() as i64
                            {
                                lazy_init!(users_deactive => users_deactive.push(session.clone()), vec![session.clone()]);
                            }
                        }
                    }

                    if let Some(users_deactive) = users_deactive {
                        {
                            let user_sessions =
                                user_sessions_service.user_sessions();
                            let mut indexes = user_sessions.write().await;

                            for session in users_deactive.iter() {
                                user_sessions.delete_inner(
                                    &mut indexes,
                                    &session.user_id,
                                    &session.username.load(),
                                    &session.id,
                                    session
                                        .username_unicode
                                        .load()
                                        .as_deref()
                                        .map(|s| s.as_str()),
                                );
                            }
                        }

                        info!(
                            target: LOG_TARGET,
                            "Done in: {:?} ({} sessions cleared)",
                            start.elapsed(),
                            users_deactive.len()
                        );
                    } else {
                        info!(
                            target: LOG_TARGET,
                            "Done in: {:?} (0 sessions cleared)",
                            start.elapsed(),
                        );
                    }
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
pub struct CliBanchoStateBackgroundServiceConfigs {
    #[default(180)]
    #[arg(long, default_value = "180")]
    pub user_sessions_recycle_deactive_secs: u64,

    #[default(180)]
    #[arg(long, default_value = "180")]
    pub user_sessions_recycle_interval_secs: u64,
}

pub struct UserSessionsRecycleConfig;

impl UserSessionsRecycleConfig {
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
pub struct BanchoStateBackgroundServiceConfigs {
    pub user_sessions_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
}

impl BanchoStateBackgroundServiceConfigs {
    pub fn new(
        user_sessions_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
    ) -> Self {
        Self { user_sessions_recycle }
    }
}

#[async_trait]
impl BanchoStateBackgroundService for BanchoStateBackgroundServiceImpl {
    fn start_all(&self, configs: BanchoStateBackgroundServiceConfigs) {
        self.start_user_sessions_recycle(configs.user_sessions_recycle);
    }

    fn start_user_sessions_recycle(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    ) {
        self.tasks
            .user_sessions_recycle
            .start(self.user_sessions_recycle_factory(config.clone()), config);
    }

    fn stop_user_sessions_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        self.tasks.user_sessions_recycle.stop()
    }
}
