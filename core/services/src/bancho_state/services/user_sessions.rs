use crate::bancho_state::{
    DynUserSessionsService, Session, UserSessionsInner, UserSessionsService,
};
use async_trait::async_trait;
use peace_pb::bancho_state_rpc::UserQuery;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default, Clone)]
pub struct UserSessionsServiceImpl {
    inner: Arc<RwLock<UserSessionsInner>>,
}

impl UserSessionsServiceImpl {
    pub fn into_service(self) -> DynUserSessionsService {
        Arc::new(self) as DynUserSessionsService
    }
}

#[async_trait]
impl UserSessionsService for UserSessionsServiceImpl {
    fn inner(&self) -> Arc<RwLock<UserSessionsInner>> {
        self.inner.clone()
    }

    async fn create(&self, session: Session) -> String {
        self.inner.write().await.create(session).await
    }

    async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>> {
        self.inner.write().await.delete(query).await
    }

    async fn get(&self, query: &UserQuery) -> Option<Arc<Session>> {
        self.inner.read().await.get(query)
    }

    async fn exists(&self, query: &UserQuery) -> bool {
        self.inner.read().await.exists(query)
    }

    async fn clear(&self) {
        self.inner.write().await.clear()
    }

    async fn len(&self) -> usize {
        self.inner.read().await.len()
    }
}
