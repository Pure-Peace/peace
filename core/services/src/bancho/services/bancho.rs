use super::BanchoService;
use crate::bancho_state::DynBanchoStateService;
use bancho_packets::{server, PacketBuilder};
use peace_domain::users::Password;
use peace_pb::{
    bancho_rpc::{bancho_rpc_client::BanchoRpcClient, *},
    bancho_state_rpc::*,
};
use peace_repositories::users::DynUsersRepository;
use std::net::IpAddr;
use tonic::{async_trait, transport::Channel, Request, Response, Status};

#[derive(Clone)]
pub enum BanchoServiceImpl {
    Remote(BanchoServiceRemote),
    Local(BanchoServiceLocal),
}

#[derive(Clone)]
pub struct BanchoServiceRemote(BanchoRpcClient<Channel>);

impl BanchoServiceRemote {
    pub fn new(bancho_rpc_client: BanchoRpcClient<Channel>) -> Self {
        Self(bancho_rpc_client)
    }

    pub fn client(&self) -> BanchoRpcClient<Channel> {
        self.0.clone()
    }
}

#[derive(Clone)]
pub struct BanchoServiceLocal {
    users_repository: DynUsersRepository,
    bancho_state_service: DynBanchoStateService,
}

impl BanchoServiceLocal {
    pub fn new(
        users_repository: DynUsersRepository,
        bancho_state_service: DynBanchoStateService,
    ) -> Self {
        Self { users_repository, bancho_state_service }
    }
}

#[async_trait]
impl BanchoService for BanchoServiceImpl {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        match self {
            Self::Remote(svc) => svc.client().ping(request).await,
            Self::Local(svc) => {
                println!("Got a request: {:?}", request);

                let reply = BanchoReply { packet: None };

                Ok(Response::new(reply))
            },
        }
    }

    async fn login(
        &self,
        client_ip: IpAddr,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginSuccess>, Status> {
        match self {
            Self::Remote(svc) => svc.client().login(request).await,
            Self::Local(svc) => {
                let req = request.into_inner();

                let user = svc
                    .users_repository
                    .get_user_model_by_username(
                        Some(req.username.as_str()),
                        Some(req.username.as_str()),
                    )
                    .await?;

                Password::verify_password(
                    user.password.as_str(),
                    req.password.as_str(),
                )
                .map_err(|err| Status::unauthenticated(err.to_string()))?;

                let CreateUserSessionResponse { session_id } = svc
                    .bancho_state_service
                    .create_user_session(Request::new(
                        CreateUserSessionRequest {
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
                        },
                    ))
                    .await?
                    .into_inner();

                info!(target: "bancho.login", "user <{}:{}> logged in (session_id: {})", user.name_safe, user.id, session_id);

                Ok(Response::new(LoginSuccess {
                    session_id,
                    packet: Some(
                        PacketBuilder::new()
                            .add(server::login_reply(
                                bancho_packets::LoginResult::Success(user.id),
                            ))
                            .add(server::notification("welcome to peace!"))
                            .build(),
                    ),
                }))
            },
        }
    }

    async fn request_status_update(
        &self,
        request: Request<RequestStatusUpdateRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        match self {
            Self::Remote(svc) =>
                svc.client().request_status_update(request).await,
            Self::Local(svc) => {
                println!("Got a request: {:?}", request);

                let reply = BanchoReply { packet: None };

                Ok(Response::new(reply))
            },
        }
    }

    async fn presence_request_all(
        &self,
        request: Request<PresenceRequestAllRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        match self {
            Self::Remote(svc) =>
                svc.client().presence_request_all(request).await,
            Self::Local(svc) => {
                println!("Got a request: {:?}", request);

                let reply = BanchoReply { packet: None };

                Ok(Response::new(reply))
            },
        }
    }

    async fn spectate_stop(
        &self,
        request: Request<SpectateStopRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        match self {
            Self::Remote(svc) => svc.client().spectate_stop(request).await,
            Self::Local(svc) => {
                println!("Got a request: {:?}", request);

                let reply = BanchoReply { packet: None };

                Ok(Response::new(reply))
            },
        }
    }

    async fn spectate_cant(
        &self,
        request: Request<SpectateCantRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        match self {
            Self::Remote(svc) => svc.client().spectate_cant(request).await,
            Self::Local(svc) => {
                println!("Got a request: {:?}", request);

                let reply = BanchoReply { packet: None };

                Ok(Response::new(reply))
            },
        }
    }

    async fn lobby_part(
        &self,
        request: Request<LobbyPartRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        match self {
            Self::Remote(svc) => svc.client().lobby_part(request).await,
            Self::Local(svc) => {
                println!("Got a request: {:?}", request);

                let reply = BanchoReply { packet: None };

                Ok(Response::new(reply))
            },
        }
    }

    async fn lobby_join(
        &self,
        request: Request<LobbyJoinRequest>,
    ) -> Result<Response<BanchoReply>, Status> {
        match self {
            Self::Remote(svc) => svc.client().lobby_join(request).await,
            Self::Local(svc) => {
                println!("Got a request: {:?}", request);

                let reply = BanchoReply { packet: None };

                Ok(Response::new(reply))
            },
        }
    }
}
