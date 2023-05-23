use super::traits::*;
use crate::{bancho_state::UserSessions, IntoService};
use async_trait::async_trait;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct UserSessionsServiceImpl {
    pub user_sessions: Arc<UserSessions>,
    pub notify_queue: Arc<Mutex<MessageQueue>>,
}

impl Default for UserSessionsServiceImpl {
    fn default() -> Self {
        Self {
            user_sessions: Arc::default(),
            notify_queue: Arc::new(Mutex::new(MessageQueue {
                messages: BTreeMap::new(),
            })),
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
    fn notify_queue(&self) -> &Arc<Mutex<MessageQueue>> {
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
