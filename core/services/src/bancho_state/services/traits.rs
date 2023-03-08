use crate::bancho_state::{BanchoStateError, Session, UserSessionsInner};
use async_trait::async_trait;
use peace_pb::{bancho_state::*, base::ExecSuccess};
use std::sync::Arc;
use tokio::sync::RwLock;
use tools::async_collections::{BackgroundTask, BackgroundTaskError};

pub type DynBanchoStateService = Arc<dyn BanchoStateService + Send + Sync>;
pub type DynBanchoStateBackgroundService =
    Arc<dyn BanchoStateBackgroundService + Send + Sync>;
pub type DynUserSessionsService = Arc<dyn UserSessionsService + Send + Sync>;

#[async_trait]
pub trait BanchoStateBackgroundService {
    fn start_all(&self);
    fn start_user_sessions_recycle(&self);
    fn stop_user_sessions_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError>;
}

#[async_trait]
pub trait UserSessionsService {
    fn user_sessions(&self) -> &Arc<RwLock<UserSessionsInner>>;

    async fn create(&self, user: Session) -> Arc<Session>;

    async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>>;

    async fn get(&self, query: &UserQuery) -> Option<Arc<Session>>;

    async fn exists(&self, query: &UserQuery) -> bool;

    async fn clear(&self);

    async fn len(&self) -> usize;
}

#[async_trait]
pub trait BanchoStateService {
    async fn broadcast_bancho_packets(
        &self,
        request: BroadcastBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn enqueue_bancho_packets(
        &self,
        request: EnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn batch_enqueue_bancho_packets(
        &self,
        request: BatchEnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn dequeue_bancho_packets(
        &self,
        request: DequeueBanchoPacketsRequest,
    ) -> Result<BanchoPackets, BanchoStateError>;

    async fn batch_dequeue_bancho_packets(
        &self,
        request: BatchDequeueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn create_user_session(
        &self,
        request: CreateUserSessionRequest,
    ) -> Result<CreateUserSessionResponse, BanchoStateError>;

    async fn delete_user_session(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn check_user_session_exists(
        &self,
        query: UserQuery,
    ) -> Result<UserSessionExistsResponse, BanchoStateError>;

    async fn get_user_session(
        &self,
        query: UserQuery,
    ) -> Result<GetUserSessionResponse, BanchoStateError>;

    async fn get_user_session_with_fields(
        &self,
        raw_query: RawUserQueryWithFields,
    ) -> Result<GetUserSessionResponse, BanchoStateError>;

    async fn get_all_sessions(
        &self,
    ) -> Result<GetAllSessionsResponse, BanchoStateError>;

    async fn send_user_stats_packet(
        &self,
        request: SendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn send_all_presences(
        &self,
        request: SendAllPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}
