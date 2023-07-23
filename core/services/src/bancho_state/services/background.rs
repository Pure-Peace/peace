use super::traits::*;
use crate::bancho_state::{
    BanchoSession, DynBanchoStateBackgroundService, NotifyMessagesCleaner,
    UserSessionsCleaner,
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
        BackgroundTaskManager, CommonRecycleBackgroundTaskConfig,
        LoopBackgroundTaskConfig, SignalHandle,
    },
    atomic::{Atomic, AtomicValue, U64},
    lazy_init, Timestamp, Ulid,
};

#[derive(Clone, Default)]
pub struct Tasks {
    pub user_sessions_recycle: BackgroundTaskManager,
    pub notify_messages_recycle: BackgroundTaskManager,
}

#[derive(Clone)]
pub struct BanchoStateBackgroundServiceImpl {
    pub user_sessions_service: DynUserSessionsService,
    pub tasks: Tasks,
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

        let user_sessions_service = self.user_sessions_service.clone();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let user_sessions_service = user_sessions_service.clone();
            let cfg = config.clone();

            let task = async move {
                loop {
                    tokio::time::sleep(*cfg.loop_interval.load().as_ref())
                        .await;
                    debug!(
                        target: LOG_TARGET,
                        "user sessions recycling started!"
                    );
                    let start = Instant::now();

                    let mut sessions_deactive = None::<Vec<Arc<BanchoSession>>>;
                    // messages before this id means all users has readed
                    let mut min_notify_msg_id_in_all_users = None::<Ulid>;

                    let current_timestamp = Timestamp::now();
                    let deadline = cfg.dead.val();

                    {
                        let user_sessions =
                            user_sessions_service.user_sessions().read().await;

                        for session in user_sessions.values() {
                            if session.is_deactive(current_timestamp, deadline)
                            {
                                lazy_init!(sessions_deactive => sessions_deactive.push(session.clone()), vec![session.clone()]);
                            }

                            // update min notify msg id
                            lazy_init!(min_notify_msg_id_in_all_users, Some(val) => {
                                let notify_index = *session.extends.notify_index.val();
                                if val > notify_index {
                                    min_notify_msg_id_in_all_users = Some(notify_index);
                                }
                            }, *session.extends.notify_index.load().as_ref())
                        }
                    }

                    let removed_deactive_sessions = match sessions_deactive {
                        Some(sessions_deactive) => {
                            let user_sessions =
                                &user_sessions_service.user_sessions();

                            // remove deactive sessions
                            {
                                let mut indexes = user_sessions.write().await;
                                for session in sessions_deactive.iter() {
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
                            };

                            sessions_deactive.len()
                        },
                        None => 0,
                    };

                    // remove messages that all users has readed
                    let removed_notify_msg =
                        match min_notify_msg_id_in_all_users {
                            Some(min_msg_id) => user_sessions_service
                                .notify_queue()
                                .write()
                                .await
                                .remove_messages_before_id(&min_msg_id),
                            None => 0,
                        };

                    let end = start.elapsed();

                    debug!(
                        target: LOG_TARGET,
                        "Done in: {end:?} ({removed_deactive_sessions} sessions removed, {removed_notify_msg} old notify msg removed)",
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

    pub fn notify_messages_recycle_factory(
        &self,
        config: Arc<LoopBackgroundTaskConfig>,
    ) -> BackgroundTaskFactory {
        const LOG_TARGET: &str =
            "bancho_state::background_tasks::notify_messages_recycling";

        let user_sessions_service = self.user_sessions_service.to_owned();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let user_sessions_service = user_sessions_service.to_owned();
            let cfg = config.to_owned();

            let task = async move {
                loop {
                    tokio::time::sleep(*cfg.loop_interval.load().as_ref())
                        .await;
                    debug!(
                        target: LOG_TARGET,
                        "notify messages recycling started!"
                    );
                    let start = Instant::now();

                    let mut invalid_messages = None::<Vec<Ulid>>;

                    let removed_notify_msg = {
                        for (key, msg) in user_sessions_service
                            .notify_queue()
                            .read()
                            .await
                            .messages
                            .iter()
                        {
                            lazy_init!(invalid_messages => if !msg.is_valid() {
                                invalid_messages.push(*key);
                            }, vec![*key]);
                        }

                        match invalid_messages {
                            Some(invalid_messages) => user_sessions_service
                                .notify_queue()
                                .write()
                                .await
                                .remove_messages(&invalid_messages),
                            None => 0,
                        }
                    };

                    let end = start.elapsed();

                    debug!(
                        target: LOG_TARGET,
                        "Done in: {end:?} ({removed_notify_msg} invalid notify msg removed)",
                    );
                }
            };

            info!(
                target: LOG_TARGET,
                "Service started! (sleep={:?})",
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
    pub bancho_user_sessions_recycle_deactive_secs: u64,

    #[default(180)]
    #[arg(long, default_value = "180")]
    pub bancho_user_sessions_recycle_interval_secs: u64,

    #[default(300)]
    #[arg(long, default_value = "300")]
    pub bancho_notify_messages_recycle_interval_secs: u64,
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

    #[inline]
    pub fn buid_with_cfg(
        cfg: &CliBanchoStateBackgroundServiceConfigs,
    ) -> Arc<CommonRecycleBackgroundTaskConfig> {
        Self::build(
            cfg.bancho_user_sessions_recycle_deactive_secs,
            cfg.bancho_user_sessions_recycle_interval_secs,
        )
    }
}

pub struct NotifyMessagesRecycleConfig;

impl NotifyMessagesRecycleConfig {
    pub fn build(loop_interval: u64) -> Arc<LoopBackgroundTaskConfig> {
        LoopBackgroundTaskConfig {
            loop_interval: Atomic::new(Duration::from_secs(loop_interval)),
            manual_stop: true.into(),
        }
        .into()
    }

    #[inline]
    pub fn buid_with_cfg(
        cfg: &CliBanchoStateBackgroundServiceConfigs,
    ) -> Arc<LoopBackgroundTaskConfig> {
        Self::build(cfg.bancho_notify_messages_recycle_interval_secs)
    }
}

#[derive(Debug, Default, Clone)]
pub struct BanchoStateBackgroundServiceConfigs {
    pub user_sessions_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
    pub notify_messages_recyce: Arc<LoopBackgroundTaskConfig>,
}

impl BanchoStateBackgroundServiceConfigs {
    #[inline]
    pub fn new(
        user_sessions_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
        notify_messages_recyce: Arc<LoopBackgroundTaskConfig>,
    ) -> Self {
        Self { user_sessions_recycle, notify_messages_recyce }
    }

    #[inline]
    pub fn with_cfg(cfg: &CliBanchoStateBackgroundServiceConfigs) -> Self {
        Self {
            user_sessions_recycle: UserSessionsRecycleConfig::buid_with_cfg(
                cfg,
            ),
            notify_messages_recyce: NotifyMessagesRecycleConfig::buid_with_cfg(
                cfg,
            ),
        }
    }
}

#[async_trait]
impl BanchoStateBackgroundService for BanchoStateBackgroundServiceImpl {
    fn start_all(&self, configs: BanchoStateBackgroundServiceConfigs) {
        self.start_user_sessions_recycle(configs.user_sessions_recycle);
        self.start_notify_messages_recyce(configs.notify_messages_recyce);
    }
}

#[async_trait]
impl UserSessionsCleaner for BanchoStateBackgroundServiceImpl {
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

#[async_trait]
impl NotifyMessagesCleaner for BanchoStateBackgroundServiceImpl {
    fn start_notify_messages_recyce(
        &self,
        config: Arc<LoopBackgroundTaskConfig>,
    ) {
        self.tasks.notify_messages_recycle.start(
            self.notify_messages_recycle_factory(config.clone()),
            config,
        );
    }

    fn stop_notify_messages_recyce(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        self.tasks.notify_messages_recycle.stop()
    }
}
