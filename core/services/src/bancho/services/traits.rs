use peace_pb::bancho_rpc::*;
use std::{net::IpAddr, sync::Arc};
use tonic::{async_trait, Request, Response, Status};

pub type DynBanchoService = Arc<dyn BanchoService + Send + Sync>;

#[async_trait]
pub trait BanchoService {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<BanchoReply>, Status>;

    async fn login(
        &self,
        client_ip: IpAddr,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginSuccess>, Status>;

    async fn request_status_update(
        &self,
        request: Request<RequestStatusUpdateRequest>,
    ) -> Result<Response<BanchoReply>, Status>;

    async fn presence_request_all(
        &self,
        request: Request<PresenceRequestAllRequest>,
    ) -> Result<Response<BanchoReply>, Status>;

    async fn spectate_stop(
        &self,
        request: Request<SpectateStopRequest>,
    ) -> Result<Response<BanchoReply>, Status>;

    async fn spectate_cant(
        &self,
        request: Request<SpectateCantRequest>,
    ) -> Result<Response<BanchoReply>, Status>;

    async fn lobby_part(
        &self,
        request: Request<LobbyPartRequest>,
    ) -> Result<Response<BanchoReply>, Status>;

    async fn lobby_join(
        &self,
        request: Request<LobbyJoinRequest>,
    ) -> Result<Response<BanchoReply>, Status>;
}
