use crate::bancho_state::{
    DynUserSessionsService, User, UserSessionsInner, UserSessionsService,
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

    async fn create(&self, user: User) -> String {
        self.inner.write().await.create(user).await
    }

    async fn delete(&self, query: &UserQuery) -> Option<Arc<RwLock<User>>> {
        self.inner.write().await.delete(query).await
    }

    async fn delete_user(&self, user: &User) -> Option<Arc<RwLock<User>>> {
        self.inner.write().await.delete_user(user)
    }

    async fn get(&self, query: &UserQuery) -> Option<Arc<RwLock<User>>> {
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
