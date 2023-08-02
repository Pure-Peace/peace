use super::traits::*;
use crate::{
    bancho_state::{BanchoSessionData, UserSessions},
    IntoService,
};
use async_trait::async_trait;
use peace_snapshot::CreateSnapshot;
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
pub struct UserSessionsServiceSnapshot {
    pub user_sessions: Vec<BanchoSessionData>,
    pub notify_queue: Vec<BanchoMessageData>,
}

#[async_trait]
impl CreateSnapshot<UserSessionsServiceSnapshot> for UserSessionsServiceImpl {
    async fn create_snapshot(&self) -> UserSessionsServiceSnapshot {
        UserSessionsServiceSnapshot {
            user_sessions: self.user_sessions.create_snapshot().await,
            notify_queue: self.notify_queue.create_snapshot().await,
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
