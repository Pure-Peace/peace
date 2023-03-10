use peace_pb::{bancho_state::*, base::ExecSuccess};
use peace_services::bancho_state::DynBanchoStateService;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct BanchoStateRpcImpl {
    pub bancho_state_service: DynBanchoStateService,
}

impl BanchoStateRpcImpl {
    pub fn new(bancho_state_service: DynBanchoStateService) -> Self {
        Self { bancho_state_service }
    }
}

#[tonic::async_trait]
impl bancho_state_rpc_server::BanchoStateRpc for BanchoStateRpcImpl {
    async fn broadcast_bancho_packets(
        &self,
        request: Request<BroadcastBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .broadcast_bancho_packets(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn enqueue_bancho_packets(
        &self,
        request: Request<EnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .enqueue_bancho_packets(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn batch_enqueue_bancho_packets(
        &self,
        request: Request<BatchEnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .batch_enqueue_bancho_packets(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn dequeue_bancho_packets(
        &self,
        request: Request<DequeueBanchoPacketsRequest>,
    ) -> Result<Response<BanchoPackets>, Status> {
        self.bancho_state_service
            .dequeue_bancho_packets(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn batch_dequeue_bancho_packets(
        &self,
        request: Request<BatchDequeueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .batch_dequeue_bancho_packets(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn create_user_session(
        &self,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status> {
        self.bancho_state_service
            .create_user_session(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .delete_user_session(request.into_inner().into())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn check_user_session_exists(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status> {
        self.bancho_state_service
            .check_user_session_exists(request.into_inner().into())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn get_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        self.bancho_state_service
            .get_user_session(request.into_inner().into())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn get_user_session_with_fields(
        &self,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        self.bancho_state_service
            .get_user_session_with_fields(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn get_all_sessions(
        &self,
        _: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status> {
        self.bancho_state_service
            .get_all_sessions()
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn send_user_stats_packet(
        &self,
        request: Request<SendUserStatsPacketRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .send_user_stats_packet(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn send_all_presences(
        &self,
        request: Request<SendAllPresencesRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .send_all_presences(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn batch_send_user_stats_packet(
        &self,
        request: Request<BatchSendUserStatsPacketRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .batch_send_user_stats_packet(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn update_presence_filter(
        &self,
        request: Request<UpdatePresenceFilterRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.bancho_state_service
            .update_presence_filter(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }
}
