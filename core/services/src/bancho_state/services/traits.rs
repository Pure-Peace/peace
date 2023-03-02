use crate::bancho_state::{Session, UserSessionsInner};
use async_trait::async_trait;
use peace_pb::bancho_state_rpc::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

pub type DynBanchoStateService = Arc<dyn BanchoStateService + Send + Sync>;
pub type DynBackgroundService = Arc<dyn BackgroundService + Send + Sync>;
pub type DynUserSessionsService = Arc<dyn UserSessionsService + Send + Sync>;

#[async_trait]
pub trait BackgroundService {
    fn start(&self);
}

#[async_trait]
pub trait UserSessionsService {
    fn user_sessions(&self) -> Arc<RwLock<UserSessionsInner>>;

    async fn create(&self, user: Session) -> String;

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

    async fn create_user_session(
        &self,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status>;

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status>;

    async fn check_user_session_exists(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status>;

    async fn get_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status>;

    async fn get_user_session_with_fields(
        &self,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status>;

    async fn get_all_sessions(
        &self,
        _request: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status>;

    async fn send_user_stats_packet(
        &self,
        request: Request<SendUserStatsPacketRequest>,
    ) -> Result<Response<ExecSuccess>, Status>;
}
