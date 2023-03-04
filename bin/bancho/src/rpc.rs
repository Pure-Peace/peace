use peace_pb::bancho_rpc::*;
use peace_rpc::extensions::ClientIp;
use peace_services::bancho::DynBanchoService;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct BanchoRpcImpl {
    pub bancho_service: DynBanchoService,
}

impl BanchoRpcImpl {
    pub fn new(bancho_service: DynBanchoService) -> Self {
        Self { bancho_service }
    }
}

#[tonic::async_trait]
impl bancho_rpc_server::BanchoRpc for BanchoRpcImpl {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .ping(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginSuccess>, Status> {
        let client_ip = ClientIp::from_request(&request)?;
        self.bancho_service
            .login(client_ip.into(), request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn request_status_update(
        &self,
        request: Request<RequestStatusUpdateRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .request_status_update(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn presence_request_all(
        &self,
        request: Request<PresenceRequestAllRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .presence_request_all(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn request_stats(
        &self,
        request: Request<StatsRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .request_stats(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn change_action(
        &self,
        request: Request<ChangeActionRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .change_action(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn receive_updates(
        &self,
        request: Request<ReceiveUpdatesRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .receive_updates(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn toggle_block_non_friend_dms(
        &self,
        request: Request<ToggleBlockNonFriendDmsRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .toggle_block_non_friend_dms(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn user_logout(
        &self,
        request: Request<UserLogoutRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .user_logout(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn request_presence(
        &self,
        request: Request<PresenceRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .request_presence(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn spectate_stop(
        &self,
        request: Request<SpectateStopRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .spectate_stop(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn spectate_cant(
        &self,
        request: Request<SpectateCantRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .spectate_cant(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn lobby_part(
        &self,
        request: Request<LobbyPartRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .lobby_part(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }

    async fn lobby_join(
        &self,
        request: Request<LobbyJoinRequest>,
    ) -> Result<Response<HandleCompleted>, Status> {
        self.bancho_service
            .lobby_join(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(|resp| Response::new(resp))
    }
}
