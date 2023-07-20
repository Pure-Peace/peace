use super::traits::*;
use crate::{bancho_state::UserSessions, DumpData, IntoService};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct UserSessionsServiceImpl {
    pub user_sessions: Arc<UserSessions>,
    pub notify_queue: Arc<BanchoMessageQueue>,
}

impl UserSessionsServiceImpl {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionsServiceDump {
    pub user_sessions: Vec<UserSessionData>,
    pub notify_queue: Vec<BanchoMessageData>,
}

#[async_trait]
impl DumpData<UserSessionsServiceDump> for UserSessionsServiceImpl {
    async fn dump_data(&self) -> UserSessionsServiceDump {
        UserSessionsServiceDump {
            user_sessions: self.user_sessions.dump_sessions().await,
            notify_queue: self.notify_queue.dump_messages().await,
        }
    }
}

impl Default for UserSessionsServiceImpl {
    fn default() -> Self {
        Self {
            user_sessions: Arc::new(UserSessions::new()),
            notify_queue: Arc::new(BanchoMessageQueue::default()),
        }
    }
}

impl IntoService<DynUserSessionsService> for UserSessionsServiceImpl {
    #[inline]
    fn into_service(self) -> DynUserSessionsService {
        Arc::new(self) as DynUserSessionsService
    }
}

impl UserSessionsStore for UserSessionsServiceImpl {
    #[inline]
    fn user_sessions(&self) -> &Arc<UserSessions> {
        &self.user_sessions
    }
}

impl NotifyMessagesQueue for UserSessionsServiceImpl {
    #[inline]
    fn notify_queue(&self) -> &Arc<BanchoMessageQueue> {
        &self.notify_queue
    }
}

#[async_trait]
impl UserSessionsCount for UserSessionsServiceImpl {}

#[async_trait]
impl UserSessionsClear for UserSessionsServiceImpl {}

#[async_trait]
impl UserSessionsGet for UserSessionsServiceImpl {}

#[async_trait]
impl UserSessionsDelete for UserSessionsServiceImpl {}

#[async_trait]
impl UserSessionsCreate for UserSessionsServiceImpl {}

#[async_trait]
impl UserSessionsExists for UserSessionsServiceImpl {}

#[async_trait]
impl UserSessionsService for UserSessionsServiceImpl {}
