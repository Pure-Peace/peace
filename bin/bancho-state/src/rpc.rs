use peace_pb::bancho_state_rpc::*;
use peace_services::bancho_state::{
    DynBackgroundService, DynBanchoStateService,
};
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct BanchoStateRpcImpl {
    pub bancho_state_service: DynBanchoStateService,
    pub background_service: DynBackgroundService,
}

impl BanchoStateRpcImpl {
    pub fn new(
        bancho_state_service: DynBanchoStateService,
        background_service: DynBackgroundService,
    ) -> Self {
        Self { bancho_state_service, background_service }
    }
}

#[tonic::async_trait]
impl bancho_state_rpc_server::BanchoStateRpc for BanchoStateRpcImpl {
    async fn broadcast_bancho_packets(
        &self,
        request: Request<BroadcastBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service.broadcast_bancho_packets(request).await
    }

    async fn enqueue_bancho_packets(
        &self,
        request: Request<EnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service.enqueue_bancho_packets(request).await
    }

    async fn batch_enqueue_bancho_packets(
        &self,
        request: Request<BatchEnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service.batch_enqueue_bancho_packets(request).await
    }

    async fn dequeue_bancho_packets(
        &self,
        request: Request<DequeueBanchoPacketsRequest>,
    ) -> Result<Response<BanchoPackets>, Status> {
        self.bancho_state_service.dequeue_bancho_packets(request).await
    }

    async fn batch_dequeue_bancho_packets(
        &self,
        request: Request<BatchDequeueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service.batch_dequeue_bancho_packets(request).await
    }

    async fn create_user_session(
        &self,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status> {
        self.bancho_state_service.create_user_session(request).await
    }

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service.delete_user_session(request).await
    }

    async fn check_user_session_exists(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status> {
        self.bancho_state_service.check_user_session_exists(request).await
    }

    async fn get_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        self.bancho_state_service.get_user_session(request).await
    }

    async fn get_user_session_with_fields(
        &self,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        self.bancho_state_service.get_user_session_with_fields(request).await
    }

    async fn get_all_sessions(
        &self,
        _request: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status> {
        self.bancho_state_service.get_all_sessions(_request).await
    }
}
