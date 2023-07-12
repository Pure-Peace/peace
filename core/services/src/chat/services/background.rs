use super::ChatServiceImpl;
use crate::chat::{
    Channel, ChatBackgroundService, ChatSession, DynChatBackgroundService,
};
use async_trait::async_trait;
use clap_serde_derive::ClapSerde;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tools::{
    async_collections::{
        BackgroundTaskFactory, BackgroundTaskManager,
        CommonRecycleBackgroundTaskConfig, LoopBackgroundTaskConfig,
        SignalHandle,
    },
    atomic::{Atomic, AtomicValue, U64},
    lazy_init, Timestamp, Ulid,
};

#[derive(Clone, Default)]
pub struct Tasks {
    pub user_sessions_recycle: BackgroundTaskManager,
    pub notify_messages_recycle: BackgroundTaskManager,
    pub channel_messages_recycle: BackgroundTaskManager,
}

#[derive(Clone)]
pub struct ChatBackgroundServiceImpl {
    pub chat_service: Arc<ChatServiceImpl>,
    pub tasks: Tasks,
}

impl ChatBackgroundServiceImpl {
    pub fn into_service(self) -> DynChatBackgroundService {
        Arc::new(self) as DynChatBackgroundService
    }

    pub fn new(chat_service: Arc<ChatServiceImpl>) -> Self {
        Self { chat_service, tasks: Tasks::default() }
    }

    pub fn user_sessions_recycle_factory(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    ) -> BackgroundTaskFactory {
        const LOG_TARGET: &str =
            "chat::background_tasks::user_sessions_recycling";

        let user_sessions = self.chat_service.user_sessions.clone();
        let notify_queue = self.chat_service.notify_queue.clone();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let user_sessions = user_sessions.clone();
            let notify_queue = notify_queue.clone();
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

                    let mut sessions_deactive = None::<Vec<Arc<ChatSession>>>;
                    // messages before this id means all users has readed
                    let mut min_notify_msg_id_in_all_users = None::<Ulid>;

                    let current_timestamp = Timestamp::now();
                    let deadline = cfg.dead.val();

                    {
                        let user_sessions = user_sessions.read().await;

                        for session in user_sessions.values() {
                            if session.is_deactive(current_timestamp, deadline)
                            {
                                lazy_init!(sessions_deactive => sessions_deactive.push(session.clone()), vec![session.clone()]);
                            }

                            // update min notify msg id
                            if let Some(bancho_ext) =
                                session.extends.bancho_ext.load().as_ref()
                            {
                                lazy_init!(min_notify_msg_id_in_all_users, Some(val) => {
                                    let notify_index = *bancho_ext.notify_index.val();
                                    if val > notify_index {
                                        min_notify_msg_id_in_all_users = Some(notify_index);
                                    }
                                }, *bancho_ext.notify_index.load().as_ref())
                            }
                        }
                    }

                    let removed_deactive_sessions = match sessions_deactive {
                        Some(sessions_deactive) => {
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

                            sessions_deactive.len()
                        },
                        None => 0,
                    };

                    // remove messages that all users has readed
                    let removed_notify_msg =
                        match min_notify_msg_id_in_all_users {
                            Some(min_msg_id) => notify_queue
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
            "chat::background_tasks::notify_messages_recycling";

        let notify_queue = self.chat_service.notify_queue.clone();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let notify_queue = notify_queue.clone();
            let cfg = config.clone();

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
                        for (key, msg) in
                            notify_queue.read().await.messages.iter()
                        {
                            lazy_init!(invalid_messages => if !msg.is_valid() {
                                invalid_messages.push(*key);
                            }, vec![*key]);
                        }

                        match invalid_messages {
                            Some(invalid_messages) => notify_queue
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

    pub fn channel_messages_recycle_factory(
        &self,
        config: Arc<LoopBackgroundTaskConfig>,
    ) -> BackgroundTaskFactory {
        const LOG_TARGET: &str =
            "chat::background_tasks::channel_messages_recycling";

        let channels = self.chat_service.channels.clone();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let channels = channels.clone();
            let cfg = config.clone();

            let task = async move {
                loop {
                    tokio::time::sleep(*cfg.loop_interval.load().as_ref())
                        .await;
                    debug!(
                        target: LOG_TARGET,
                        "channel messages recycling started!"
                    );
                    let mut removed_messages = 0;
                    let start = Instant::now();

                    let channels = {
                        channels
                            .read()
                            .await
                            .values()
                            .cloned()
                            .collect::<Vec<Arc<Channel>>>()
                    };

                    for channel in channels {
                        if let Some(channel_min_msg_id) =
                            channel.min_msg_index.load().as_deref()
                        {
                            let mut message_queue =
                                channel.message_queue.write().await;

                            removed_messages +=
                                message_queue.remove_invalid_messages();

                            removed_messages += message_queue
                                .remove_messages_before_id(channel_min_msg_id);
                        }
                    }

                    let end = start.elapsed();
                    debug!(
                        target: LOG_TARGET,
                        "Done in: {end:?} ({removed_messages} messages removed)",
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
pub struct CliChatBackgroundServiceConfigs {
    #[default(180)]
    #[arg(long, default_value = "180")]
    pub user_sessions_recycle_deactive_secs: u64,

    #[default(180)]
    #[arg(long, default_value = "180")]
    pub user_sessions_recycle_interval_secs: u64,

    #[default(300)]
    #[arg(long, default_value = "300")]
    pub notify_messages_recycle_interval_secs: u64,

    #[default(300)]
    #[arg(long, default_value = "300")]
    pub channel_messages_recycle_interval_secs: u64,
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
        cfg: &CliChatBackgroundServiceConfigs,
    ) -> Arc<CommonRecycleBackgroundTaskConfig> {
        Self::build(
            cfg.user_sessions_recycle_deactive_secs,
            cfg.user_sessions_recycle_interval_secs,
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
        cfg: &CliChatBackgroundServiceConfigs,
    ) -> Arc<LoopBackgroundTaskConfig> {
        Self::build(cfg.notify_messages_recycle_interval_secs)
    }
}

pub struct ChannelMessagesRecycleConfig;

impl ChannelMessagesRecycleConfig {
    pub fn build(loop_interval: u64) -> Arc<LoopBackgroundTaskConfig> {
        LoopBackgroundTaskConfig {
            loop_interval: Atomic::new(Duration::from_secs(loop_interval)),
            manual_stop: true.into(),
        }
        .into()
    }

    #[inline]
    pub fn buid_with_cfg(
        cfg: &CliChatBackgroundServiceConfigs,
    ) -> Arc<LoopBackgroundTaskConfig> {
        Self::build(cfg.notify_messages_recycle_interval_secs)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChatBackgroundServiceConfigs {
    pub user_sessions_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
    pub notify_messages_recyce: Arc<LoopBackgroundTaskConfig>,
    pub channel_messages_recyce: Arc<LoopBackgroundTaskConfig>,
}

impl ChatBackgroundServiceConfigs {
    #[inline]
    pub fn new(
        user_sessions_recycle: Arc<CommonRecycleBackgroundTaskConfig>,
        notify_messages_recyce: Arc<LoopBackgroundTaskConfig>,
        channel_messages_recyce: Arc<LoopBackgroundTaskConfig>,
    ) -> Self {
        Self {
            user_sessions_recycle,
            notify_messages_recyce,
            channel_messages_recyce,
        }
    }

    #[inline]
    pub fn with_cfg(cfg: &CliChatBackgroundServiceConfigs) -> Self {
        Self {
            user_sessions_recycle: UserSessionsRecycleConfig::buid_with_cfg(
                cfg,
            ),
            notify_messages_recyce: NotifyMessagesRecycleConfig::buid_with_cfg(
                cfg,
            ),
            channel_messages_recyce:
                ChannelMessagesRecycleConfig::buid_with_cfg(cfg),
        }
    }
}

#[async_trait]
impl ChatBackgroundService for ChatBackgroundServiceImpl {
    fn start_all(&self, configs: ChatBackgroundServiceConfigs) {
        self.tasks.user_sessions_recycle.start(
            self.user_sessions_recycle_factory(
                configs.user_sessions_recycle.clone(),
            ),
            configs.user_sessions_recycle,
        );

        self.tasks.notify_messages_recycle.start(
            self.notify_messages_recycle_factory(
                configs.notify_messages_recyce.clone(),
            ),
            configs.notify_messages_recyce,
        );

        self.tasks.channel_messages_recycle.start(
            self.channel_messages_recycle_factory(
                configs.channel_messages_recyce.clone(),
            ),
            configs.channel_messages_recyce,
        );
    }
}
