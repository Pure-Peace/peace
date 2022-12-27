use peace_pb::services::bancho::{
    bancho_rpc_server::BanchoRpc, CommonReply, LobbyJoinRequest,
    LobbyPartRequest, LoginReply, LoginRequest, PingRequest,
    PresenceRequestAllRequest, RequestStatusUpdateRequest, RpcStatus,
    SpectateCantRequest, SpectateStopRequest,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct Bancho {}

#[tonic::async_trait]
impl BanchoRpc for Bancho {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CommonReply {
            status: RpcStatus::Ok.into(),
            packet: None,
            err: None,
        };

        Ok(Response::new(reply))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = LoginReply {
            status: RpcStatus::Ok.into(),
            token: None,
            protocol: None,
            packet: None,
        };

        Ok(Response::new(reply))
    }

    async fn request_status_update(
        &self,
        request: Request<RequestStatusUpdateRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CommonReply {
            status: RpcStatus::Ok.into(),
            packet: None,
            err: None,
        };

        Ok(Response::new(reply))
    }

    async fn presence_request_all(
        &self,
        request: Request<PresenceRequestAllRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CommonReply {
            status: RpcStatus::Ok.into(),
            packet: None,
            err: None,
        };

        Ok(Response::new(reply))
    }

    async fn spectate_stop(
        &self,
        request: Request<SpectateStopRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CommonReply {
            status: RpcStatus::Ok.into(),
            packet: None,
            err: None,
        };

        Ok(Response::new(reply))
    }

    async fn spectate_cant(
        &self,
        request: Request<SpectateCantRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CommonReply {
            status: RpcStatus::Ok.into(),
            packet: None,
            err: None,
        };

        Ok(Response::new(reply))
    }

    async fn lobby_part(
        &self,
        request: Request<LobbyPartRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CommonReply {
            status: RpcStatus::Ok.into(),
            packet: None,
            err: None,
        };

        Ok(Response::new(reply))
    }

    async fn lobby_join(
        &self,
        request: Request<LobbyJoinRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = CommonReply {
            status: RpcStatus::Ok.into(),
            packet: None,
            err: None,
        };

        Ok(Response::new(reply))
    }
}
