use crate::Bancho;
use peace_pb::bancho_rpc::*;
use peace_rpc::extensions::ClientIp;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl bancho_rpc_server::BanchoRpc for Bancho {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        self.bancho_service.ping(request).await
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginSuccess>, Status> {
        let client_ip = ClientIp::from_request(&request)?;
        self.bancho_service.login(client_ip.into(), request).await
    }

    async fn request_status_update(
        &self,
        request: Request<RequestStatusUpdateRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        self.bancho_service.request_status_update(request).await
    }

    async fn presence_request_all(
        &self,
        request: Request<PresenceRequestAllRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        self.bancho_service.presence_request_all(request).await
    }

    async fn spectate_stop(
        &self,
        request: Request<SpectateStopRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        self.bancho_service.spectate_stop(request).await
    }

    async fn spectate_cant(
        &self,
        request: Request<SpectateCantRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        self.bancho_service.spectate_cant(request).await
    }

    async fn lobby_part(
        &self,
        request: Request<LobbyPartRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        self.bancho_service.lobby_part(request).await
    }

    async fn lobby_join(
        &self,
        request: Request<LobbyJoinRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        self.bancho_service.lobby_join(request).await
    }
}
