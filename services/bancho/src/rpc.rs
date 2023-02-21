use bancho_packets::{server, PacketBuilder};
use peace_db::{peace::Repository, DatabaseConnection};
use peace_pb::services::{
    bancho_rpc::{bancho_rpc_server::BanchoRpc, *},
    bancho_state_rpc::{bancho_state_rpc_client::BanchoStateRpcClient, *},
};
use peace_rpc::extensions::ClientIp;
use tonic::{transport::Channel, Request, Response, Status};

#[derive(Debug, Clone)]
pub struct Bancho {
    pub state_rpc: BanchoStateRpcClient<Channel>,
    pub db_conn: DatabaseConnection,
}

impl Bancho {
    pub fn new(
        state_rpc: BanchoStateRpcClient<Channel>,
        db_conn: DatabaseConnection,
    ) -> Self {
        Self { state_rpc, db_conn }
    }
}

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
        let client_ip = ClientIp::from_request(&request)?;
        let req = request.into_inner();

        let user_id = 1;

        let mut state = self.state_rpc.clone();

        let resp = state
            .create_user_session(Request::new(CreateUserSessionRequest {
                user_id,
                username: req.username.to_owned(),
                username_unicode: None,
                privileges: 1,
                connection_info: Some(ConnectionInfo {
                    ip: client_ip.to_string(),
                    region: "".into(),
                    latitude: 0.,
                    longitude: 0.,
                }),
            }))
            .await?
            .into_inner();

        info!(target: "bancho.login", "user <{}:{user_id}> logged in (session_id: {})", req.username, resp.session_id);

        Ok(Response::new(LoginReply {
            session_id: Some(resp.session_id),
            packet: Some(
                PacketBuilder::new()
                    .add(server::login_reply(
                        bancho_packets::LoginResult::Success(user_id),
                    ))
                    .build(),
            ),
        }))
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
