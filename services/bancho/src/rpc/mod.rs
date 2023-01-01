use peace_pb::services::bancho::{
    bancho_rpc_server::BanchoRpc, BanchoReply, LobbyJoinRequest,
    LobbyPartRequest, LoginReply, LoginRequest, PingRequest,
    PresenceRequestAllRequest, RequestStatusUpdateRequest, SpectateCantRequest,
    SpectateStopRequest,
};
use peace_rpc::extensions::ClientIp;
use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct Bancho {}

#[tonic::async_trait]
impl BanchoRpc for Bancho {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = BanchoReply { packet: None };

        Ok(Response::new(reply))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginReply>, Status> {
        let client_ip = request
            .extensions()
            .get::<ClientIp>()
            .ok_or(Status::internal("No client ip"))?;

        println!("Got a request: {:?}, ip: {:?}", request, client_ip);

        let reply = LoginReply { token: None, packet: None };

        Ok(Response::new(reply))
    }

    async fn request_status_update(
        &self,
        request: Request<RequestStatusUpdateRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = BanchoReply { packet: None };

        Ok(Response::new(reply))
    }

    async fn presence_request_all(
        &self,
        request: Request<PresenceRequestAllRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = BanchoReply { packet: None };

        Ok(Response::new(reply))
    }

    async fn spectate_stop(
        &self,
        request: Request<SpectateStopRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = BanchoReply { packet: None };

        Ok(Response::new(reply))
    }

    async fn spectate_cant(
        &self,
        request: Request<SpectateCantRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = BanchoReply { packet: None };

        Ok(Response::new(reply))
    }

    async fn lobby_part(
        &self,
        request: Request<LobbyPartRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = BanchoReply { packet: None };

        Ok(Response::new(reply))
    }

    async fn lobby_join(
        &self,
        request: Request<LobbyJoinRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = BanchoReply { packet: None };

        Ok(Response::new(reply))
    }
}
