use crate::UserSessions;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type DynAppStateRepository = Arc<dyn AppStateRepository + Send + Sync>;

#[async_trait]
pub trait AppStateRepository {
    fn user_sessions(&self) -> Arc<RwLock<UserSessions>>;
}

#[derive(Debug, Default, Clone)]
pub struct AppStateRepositoryImpl {
    /// The collection of user sessions currently active on the server.
    pub user_sessions: Arc<RwLock<UserSessions>>,
}

impl AppStateRepository for AppStateRepositoryImpl {
    fn user_sessions(&self) -> Arc<RwLock<UserSessions>> {
        self.user_sessions.clone()
    }
}
