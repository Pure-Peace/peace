use super::BanchoStateBackgroundService;
use crate::bancho_state::{DynBanchoStateBackgroundService, UserSessionsInner};
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use bancho_packets::server;
use chrono::Utc;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tools::async_collections::{
    BackgroundTask, BackgroundTaskError, BackgroundTaskFactory, SignalHandle,
};

const DEACTIVE: i64 = 20;
const SLEEP: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct Tasks {
    user_sessions_recycle: Arc<ArcSwapOption<BackgroundTask>>,
}

#[derive(Clone)]
pub struct BanchoStateBackgroundServiceImpl {
    user_sessions_inner: Arc<RwLock<UserSessionsInner>>,
    tasks: Tasks,
}

impl BanchoStateBackgroundServiceImpl {
    pub fn into_service(self) -> DynBanchoStateBackgroundService {
        Arc::new(self) as DynBanchoStateBackgroundService
    }

    pub fn new(user_sessions_inner: Arc<RwLock<UserSessionsInner>>) -> Self {
        Self {
            user_sessions_inner,
            tasks: Tasks {
                user_sessions_recycle: Arc::new(ArcSwapOption::empty()),
            },
        }
    }

    pub fn user_sessions_recycle_factory(&self) -> BackgroundTaskFactory {
        let user_sessions = self.user_sessions_inner.to_owned();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let user_sessions = user_sessions.to_owned();
            let task = async move {
                loop {
                    tokio::time::sleep(SLEEP).await;
                    info!(target: "user_sessions_recycling", "user sessions recycling task started");
                    let start = Instant::now();

                    let current_timestamp = Utc::now().timestamp();
                    let users_deactive = {
                        let mut users_deactive = Vec::new();

                        let user_sessions = user_sessions.read().await;

                        for session in
                            user_sessions.indexed_by_session_id.values()
                        {
                            let user = session.user.read().await;
                            if current_timestamp - user.last_active.timestamp()
                                > DEACTIVE
                            {
                                users_deactive.push((
                                    user.id,
                                    user.username.to_owned(),
                                    session.id.to_owned(),
                                    user.username_unicode.to_owned(),
                                ));
                            }
                        }
                        users_deactive
                    };

                    let () = {
                        let mut user_sessions = user_sessions.write().await;

                        for (user_id, username, session_id, username_unicode) in
                            users_deactive.iter()
                        {
                            user_sessions.delete_inner(
                                user_id,
                                username,
                                session_id,
                                username_unicode.as_ref().map(|s| s.as_str()),
                            );
                        }
                    };

                    let () = {
                        let user_sessions = user_sessions.read().await;

                        for (user_id, ..) in users_deactive.iter() {
                            let logout_notify =
                                Arc::new(server::user_logout(*user_id));

                            for session in
                                user_sessions.indexed_by_session_id.values()
                            {
                                session
                                    .push_packet(logout_notify.clone())
                                    .await;
                            }
                        }
                    };

                    info!(target: "user_sessions_recycling",
                        "user sessions recycling task done in {:?}",
                        start.elapsed()
                    );
                }
            };

            Box::pin(async move {
                info!(target: "user_sessions_recycling", "User sessions recycling service started!");
                tokio::select!(
                    _ = task => {},
                    _ = stop.wait_signal() => {}
                );
                warn!(target: "user_sessions_recycling", "User sessions recycling service stopped!");
            })
        }))
    }
}

#[async_trait]
impl BanchoStateBackgroundService for BanchoStateBackgroundServiceImpl {
    fn start_all(&self) {
        self.start_user_sessions_recycle();
    }

    fn start_user_sessions_recycle(&self) {
        if self.tasks.user_sessions_recycle.load().is_some() {
            return;
        }

        self.tasks.user_sessions_recycle.store(Some(Arc::new(
            BackgroundTask::start(self.user_sessions_recycle_factory(), true),
        )));
    }

    fn stop_user_sessions_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        if let Some(task) = self.tasks.user_sessions_recycle.load_full() {
            task.trigger_signal()?;
            return Ok(self.tasks.user_sessions_recycle.swap(None));
        }

        Ok(None)
    }
}
