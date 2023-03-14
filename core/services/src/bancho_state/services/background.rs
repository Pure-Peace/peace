use super::BanchoStateBackgroundService;
use crate::bancho_state::{
    DynBanchoStateBackgroundService, DynUserSessionsService, UserSessions,
};
use async_trait::async_trait;
use chrono::Utc;
use peace_pb::bancho_state::UserQuery;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tools::{
    async_collections::{
        BackgroundTask, BackgroundTaskError, BackgroundTaskFactory,
        BackgroundTaskManager, SignalHandle,
    },
    atomic::AtomicValue,
};

const DEACTIVE: i64 = 180;
const SLEEP: Duration = Duration::from_secs(180);

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

    pub fn user_sessions_recycle_factory(&self) -> BackgroundTaskFactory {
        const LOG_TARGET: &str =
            "bancho_state::background_tasks::user_sessions_recycling";

        #[inline]
        async fn collect_deactive_users(
            user_sessions: &Arc<UserSessions>,
            current_timestamp: i64,
        ) -> Vec<UserQuery> {
            let mut users_deactive = Vec::new();
            let user_sessions = user_sessions.read().await;

            for session in user_sessions.values() {
                if current_timestamp - session.last_active.val() > DEACTIVE {
                    users_deactive.push(UserQuery::UserId(session.user_id));
                }
            }

            users_deactive
        }

        let user_sessions_service = self.user_sessions_service.to_owned();

        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let user_sessions_service = user_sessions_service.to_owned();

            let task = async move {
                loop {
                    tokio::time::sleep(SLEEP).await;
                    info!(target: LOG_TARGET, "Task started!");
                    let start = Instant::now();

                    let users_deactive = collect_deactive_users(
                        user_sessions_service.user_sessions(),
                        Utc::now().timestamp(),
                    )
                    .await;

                    for query in users_deactive.iter() {
                        user_sessions_service.delete(&query).await;
                    }

                    info!(
                        target: LOG_TARGET,
                        "Done in: {:?} ({} sessions cleared)",
                        start.elapsed(),
                        users_deactive.len()
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
impl BanchoStateBackgroundService for BanchoStateBackgroundServiceImpl {
    fn start_all(&self) {
        self.start_user_sessions_recycle();
    }

    fn start_user_sessions_recycle(&self) {
        self.tasks
            .user_sessions_recycle
            .start(self.user_sessions_recycle_factory(), true);
    }

    fn stop_user_sessions_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        self.tasks.user_sessions_recycle.stop()
    }
}
