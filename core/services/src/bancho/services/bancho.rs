use super::{BanchoService, DynBanchoService};
use crate::{
    bancho::{BanchoServiceError, LoginError},
    bancho_state::DynBanchoStateService,
};
use bancho_packets::{server, PacketBuilder};
use peace_domain::users::Password;
use peace_pb::{
    bancho_rpc::{bancho_rpc_client::BanchoRpcClient, *},
    bancho_state_rpc::*,
};
use peace_repositories::users::DynUsersRepository;
use std::{net::IpAddr, sync::Arc};
use tonic::{async_trait, transport::Channel, Request, Response};

#[derive(Clone)]
pub enum BanchoServiceImpl {
    Remote(BanchoServiceRemote),
    Local(BanchoServiceLocal),
}

impl BanchoServiceImpl {
    pub fn into_service(self) -> DynBanchoService {
        Arc::new(self) as DynBanchoService
    }

    pub fn remote(client: BanchoRpcClient<Channel>) -> Self {
        Self::Remote(BanchoServiceRemote(client))
    }

    pub fn local(
        users_repository: DynUsersRepository,
        bancho_state_service: DynBanchoStateService,
    ) -> Self {
        Self::Local(BanchoServiceLocal::new(
            users_repository,
            bancho_state_service,
        ))
    }
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
    ) -> Result<Response<HandleCompleted>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .ping(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(svc) => {
                let _ = svc
                    .bancho_state_service
                    .check_user_session_exists(Request::new(
                        UserQuery::SessionId(request.into_inner().session_id)
                            .into(),
                    ))
                    .await;

                Ok(Response::new(HandleCompleted {}))
            },
        }
    }

    async fn login(
        &self,
        client_ip: IpAddr,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginSuccess>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .login(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(svc) => {
                let LoginRequest { username, password, client_version, .. } =
                    request.into_inner();
                info!("Receive login request: {username} [{client_version}] ({client_ip})");

                let user = svc
                    .users_repository
                    .get_user_model_by_username(
                        Some(username.as_str()),
                        Some(username.as_str()),
                    )
                    .await?;

                Password::verify_password(
                    user.password.as_str(),
                    password.as_str(),
                )
                .map_err(LoginError::PasswordError)?;

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
    ) -> Result<Response<HandleCompleted>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .request_status_update(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(svc) => {
                let RequestStatusUpdateRequest { session_id } =
                    request.into_inner();

                let _resp = svc
                    .bancho_state_service
                    .send_user_stats_packet(Request::new(
                        SendUserStatsPacketRequest {
                            user_query: Some(
                                UserQuery::SessionId(session_id.to_owned())
                                    .into(),
                            ),
                            to: Some(
                                BanchoPacketTarget::SessionId(session_id)
                                    .into(),
                            ),
                        },
                    ))
                    .await?;

                Ok(Response::new(HandleCompleted {}))
            },
        }
    }

    async fn presence_request_all(
        &self,
        request: Request<PresenceRequestAllRequest>,
    ) -> Result<Response<HandleCompleted>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .presence_request_all(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(Response::new(HandleCompleted {}))
            },
        }
    }

    async fn spectate_stop(
        &self,
        request: Request<SpectateStopRequest>,
    ) -> Result<Response<HandleCompleted>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .spectate_stop(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(Response::new(HandleCompleted {}))
            },
        }
    }

    async fn spectate_cant(
        &self,
        request: Request<SpectateCantRequest>,
    ) -> Result<Response<HandleCompleted>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .spectate_cant(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(Response::new(HandleCompleted {}))
            },
        }
    }

    async fn lobby_part(
        &self,
        request: Request<LobbyPartRequest>,
    ) -> Result<Response<HandleCompleted>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .lobby_part(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(Response::new(HandleCompleted {}))
            },
        }
    }

    async fn lobby_join(
        &self,
        request: Request<LobbyJoinRequest>,
    ) -> Result<Response<HandleCompleted>, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .lobby_join(request)
                .await
                .map_err(BanchoServiceError::RpcError),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(Response::new(HandleCompleted {}))
            },
        }
    }
}
