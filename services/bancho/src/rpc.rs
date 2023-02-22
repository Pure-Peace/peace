use crate::BanchoStateRpc;
use bancho_packets::{server, LoginFailedResaon, PacketBuilder};
use peace_db::peace::Repository;
use peace_domain::peace::{Ascii, Password, Unicode, Username};
use peace_pb::services::{
    bancho_rpc::{bancho_rpc_server::BanchoRpc, *},
    bancho_state_rpc::*,
};
use peace_rpc::extensions::ClientIp;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct Bancho {
    pub state_rpc: BanchoStateRpc,
    pub repo: Repository,
}

impl Bancho {
    pub fn new(state_rpc: BanchoStateRpc, repo: Repository) -> Self {
        Self { state_rpc, repo }
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

        let username = Username::<Ascii>::from_str(req.username.as_str())
            .ok()
            .map(|n| n.safe_name());
        let username_unicode =
            Username::<Unicode>::from_str(req.username.as_str())
                .ok()
                .map(|n| n.safe_name());

        let user = self
            .repo
            .find_user_by_username(username, username_unicode)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        if user.is_none() {
            return Ok(Response::new(LoginReply {
                session_id: None,
                packet: Some(
                    PacketBuilder::new()
                        .add(server::login_reply(
                            bancho_packets::LoginResult::Failed(
                                LoginFailedResaon::InvalidCredentials,
                            ),
                        ))
                        .build(),
                ),
            }));
        }

        let user = user.unwrap();
        if !Password::verify_password(
            user.password.as_str(),
            req.password.as_str(),
        )
        .map_err(|err| Status::internal(err.to_string()))?
        {
            return Ok(Response::new(LoginReply {
                session_id: None,
                packet: Some(
                    PacketBuilder::new()
                        .add(server::login_reply(
                            bancho_packets::LoginResult::Failed(
                                LoginFailedResaon::InvalidCredentials,
                            ),
                        ))
                        .build(),
                ),
            }));
        }

        let mut state = self.state_rpc.clone();
        let resp = state
            .create_user_session(Request::new(CreateUserSessionRequest {
                user_id: user.id,
                username: user.name,
                username_unicode: user.name_unicode,
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

        info!(target: "bancho.login", "user <{}:{}> logged in (session_id: {})", user.name_safe, user.id, resp.session_id);

        Ok(Response::new(LoginReply {
            session_id: Some(resp.session_id),
            packet: Some(
                PacketBuilder::new()
                    .add(server::login_reply(
                        bancho_packets::LoginResult::Success(user.id),
                    ))
                    .add(server::notification("welcome to peace!"))
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
