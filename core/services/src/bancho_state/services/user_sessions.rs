use crate::bancho_state::{
    DynUserSessionsService, Session, UserSessionsInner, UserSessionsService,
};
use async_trait::async_trait;
use peace_pb::bancho_state::UserQuery;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};

#[derive(Debug, Default, Clone)]
pub struct UserSessionsServiceImpl {
    user_sessions: Arc<RwLock<UserSessionsInner>>,
}

impl UserSessionsServiceImpl {
    pub fn into_service(self) -> DynUserSessionsService {
        Arc::new(self) as DynUserSessionsService
    }
}

#[async_trait]
impl UserSessionsService for UserSessionsServiceImpl {
    fn user_sessions(&self) -> &Arc<RwLock<UserSessionsInner>> {
        &self.user_sessions
    }

    async fn create(&self, session: Session) -> Arc<Session> {
        self.user_sessions.write().await.create(session).await
    }

    async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>> {
        self.user_sessions.write().await.delete(query).await
    }

    async fn get(&self, query: &UserQuery) -> Option<Arc<Session>> {
        self.user_sessions.read().await.get(query)
    }

    async fn exists(&self, query: &UserQuery) -> bool {
        self.user_sessions.read().await.exists(query)
    }

    async fn clear(&self) {
        self.user_sessions.write().await.clear()
    }

    async fn len(&self) -> usize {
        self.user_sessions.read().await.len()
    }
}
