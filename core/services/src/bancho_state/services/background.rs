use super::BanchoStateBackgroundService;
use crate::bancho_state::{DynBanchoStateBackgroundService, UserSessionsInner};
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use chrono::Utc;
use peace_pb::bancho_state_rpc::UserQuery;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tools::async_collections::{
    BackgroundTask, BackgroundTaskError, BackgroundTaskFactory, SignalHandle,
};

const DEACTIVE: i64 = 20;
const SLEEP: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct BanchoStateBackgroundServiceImpl {
    pub user_sessions_inner: Arc<RwLock<UserSessionsInner>>,
    pub user_sessions_recycle: Arc<ArcSwapOption<BackgroundTask>>,
}

impl BanchoStateBackgroundServiceImpl {
    pub fn into_service(self) -> DynBanchoStateBackgroundService {
        Arc::new(self) as DynBanchoStateBackgroundService
    }

    pub fn new(user_sessions_inner: Arc<RwLock<UserSessionsInner>>) -> Self {
        Self {
            user_sessions_inner,
            user_sessions_recycle: Arc::new(ArcSwapOption::empty()),
        }
    }

    pub fn user_sessions_recycle_factory(
        user_sessions: Arc<RwLock<UserSessionsInner>>,
    ) -> BackgroundTaskFactory {
        BackgroundTaskFactory::new(Arc::new(move |stop: SignalHandle| {
            let user_sessions = user_sessions.to_owned();

            Box::pin(async move {
                info!("User sessions recycling service started!");
                tokio::select!(
                    _ = async {
                        loop {
                            tokio::time::sleep(SLEEP).await;
                            info!("recycling task started");

                            let current_timestamp = Utc::now().timestamp();
                            let mut user_sessions = user_sessions.write().await;

                            let mut users_deactive = Vec::new();
                            for session in user_sessions.indexed_by_session_id.values() {
                                let user = session.user.read().await;
                                if current_timestamp - user.last_active.timestamp()
                                    > DEACTIVE
                                {
                                    users_deactive.push(user.id)
                                }
                            }

                            for user_id in users_deactive.iter() {
                                user_sessions.delete(&UserQuery::UserId(*user_id)).await;
                            }

                            info!("recycling task done: {users_deactive:?}");
                        }
                    } => {},
                    _ = stop.wait_signal() => {}
                );
                warn!("User sessions recycling service stopped!");
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
        if self.user_sessions_recycle.load().is_some() {
            return;
        }

        self.user_sessions_recycle.store(Some(Arc::new(
            BackgroundTask::start(
                BanchoStateBackgroundServiceImpl::user_sessions_recycle_factory(
                    self.user_sessions_inner.clone(),
                ),
                true,
            ),
        )));
    }

    fn stop_user_sessions_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        if let Some(task) = self.user_sessions_recycle.load_full() {
            task.trigger_signal()?;
            return Ok(self.user_sessions_recycle.swap(None));
        }

        Ok(None)
    }
}
