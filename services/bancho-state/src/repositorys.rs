use crate::UserSessions;
use async_trait::async_trait;
use peace_pb::services::bancho_state_rpc::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

pub type DynAppStateRepository = Arc<dyn AppStateRepository + Send + Sync>;
pub type DynPacketsRepository = Arc<dyn PacketsRepository + Send + Sync>;
pub type DynBackgroundServiceRepository =
    Arc<dyn BackgroundServiceRepository + Send + Sync>;
pub type DynSessionsRepository = Arc<dyn SessionsRepository + Send + Sync>;

#[async_trait]
pub trait AppStateRepository {
    fn user_sessions(&self) -> Arc<RwLock<UserSessions>>;
}

#[async_trait]
pub trait PacketsRepository {
    async fn broadcast_bancho_packets(
        &self,
        request: Request<BroadcastBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status>;
    async fn enqueue_bancho_packets(
        &self,
        request: Request<EnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status>;
    async fn batch_enqueue_bancho_packets(
        &self,
        request: Request<BatchEnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status>;
    async fn dequeue_bancho_packets(
        &self,
        request: Request<DequeueBanchoPacketsRequest>,
    ) -> Result<Response<BanchoPackets>, Status>;
    async fn batch_dequeue_bancho_packets(
        &self,
        request: Request<BatchDequeueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status>;
}

#[async_trait]
pub trait BackgroundServiceRepository {
    fn start_background_service(&self);
}

#[async_trait]
pub trait SessionsRepository {
    async fn create_user_session(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status>;
    async fn delete_user_session(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status>;
    async fn check_user_session_exists(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status>;
    async fn get_user_session(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status>;
    async fn get_user_session_with_fields(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status>;
    async fn get_all_sessions(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        _request: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status>;
}
