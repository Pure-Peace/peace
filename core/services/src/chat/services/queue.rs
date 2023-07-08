use crate::{
    bancho_state::Session,
    chat::{
        ChatExtend, ChatServiceError, DynQueueService, QueueService,
        UserSessions, UserSessionsStore,
    },
};
use async_trait::async_trait;
use peace_domain::bancho_state::CreateSessionDto;
use peace_pb::{
    bancho_state::UserQuery, base::ExecSuccess, chat::CreateQueueRequest,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct QueueServiceImpl {
    pub user_sessions: Arc<UserSessions>,
}

impl QueueServiceImpl {
    #[inline]
    pub fn new() -> Self {
        Self { user_sessions: Arc::new(UserSessions::new()) }
    }

    #[inline]
    pub fn into_service(self) -> DynQueueService {
        Arc::new(self) as DynQueueService
    }
}

impl Default for QueueServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueueService for QueueServiceImpl {
    async fn create_queue(
        &self,
        request: CreateQueueRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        let CreateQueueRequest {
            user_id,
            username,
            username_unicode,
            privileges,
        } = request;

        let session = Session::new(CreateSessionDto {
            user_id,
            username,
            username_unicode,
            privileges,
            initial_packets: None,
            extend: ChatExtend::default(),
        });

        self.user_sessions.create(session, false).await;

        Ok(ExecSuccess::default())
    }

    async fn remove_queue(
        &self,
        query: &UserQuery,
    ) -> Result<ExecSuccess, ChatServiceError> {
        self.user_sessions.delete(query).await;

        Ok(ExecSuccess::default())
    }
}

impl UserSessionsStore for QueueServiceImpl {
    #[inline]
    fn user_sessions(&self) -> &Arc<UserSessions> {
        &self.user_sessions
    }
}
