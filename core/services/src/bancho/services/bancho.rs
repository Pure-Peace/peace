use super::{BanchoService, DynBanchoService};
use crate::{
    bancho::{BanchoServiceError, DynPasswordService, LoginError},
    bancho_state::DynBanchoStateService,
};
use bancho_packets::{server, PacketBuilder};
use peace_pb::{
    bancho_rpc::{bancho_rpc_client::BanchoRpcClient, *},
    bancho_state_rpc::*,
};
use peace_repositories::users::DynUsersRepository;
use std::{net::IpAddr, sync::Arc};
use tonic::{async_trait, transport::Channel};
use tools::tonic_utils::RawRequest;

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
        password_service: DynPasswordService,
    ) -> Self {
        Self::Local(BanchoServiceLocal::new(
            users_repository,
            bancho_state_service,
            password_service,
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
    password_service: DynPasswordService,
}

impl BanchoServiceLocal {
    pub fn new(
        users_repository: DynUsersRepository,
        bancho_state_service: DynBanchoStateService,
        password_service: DynPasswordService,
    ) -> Self {
        Self { users_repository, bancho_state_service, password_service }
    }
}

#[async_trait]
impl BanchoService for BanchoServiceImpl {
    async fn ping(
        &self,
        request: PingRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .ping(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let _ = svc
                    .bancho_state_service
                    .check_user_session_exists(UserQuery::SessionId(
                        request.session_id,
                    ))
                    .await;

                Ok(HandleCompleted {})
            },
        }
    }

    async fn login(
        &self,
        client_ip: IpAddr,
        request: LoginRequest,
    ) -> Result<LoginSuccess, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .login(RawRequest::add_client_ip(request, client_ip))
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let LoginRequest { username, password, client_version, .. } =
                    request;
                info!("Receive login request: {username} [{client_version}] ({client_ip})");

                let user = svc
                    .users_repository
                    .get_user_by_username(
                        Some(username.as_str()),
                        Some(username.as_str()),
                    )
                    .await
                    .map_err(LoginError::UserNotExists)?;

                svc.password_service
                    .verify_password(user.password.as_str(), password.as_str())
                    .await
                    .map_err(LoginError::PasswordError)?;

                let CreateUserSessionResponse { session_id } = svc
                    .bancho_state_service
                    .create_user_session(CreateUserSessionRequest {
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
                    })
                    .await?;

                info!(target: "bancho.login", "user <{}:{}> logged in (session_id: {})", user.name_safe, user.id, session_id);

                Ok(LoginSuccess {
                    session_id,
                    packet: Some(
                        PacketBuilder::new()
                            .add(server::login_reply(
                                bancho_packets::LoginResult::Success(user.id),
                            ))
                            .add(server::notification("welcome to peace!"))
                            .build(),
                    ),
                })
            },
        }
    }

    async fn request_status_update(
        &self,
        request: RequestStatusUpdateRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .request_status_update(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let RequestStatusUpdateRequest { session_id } = request;

                let _resp = svc
                    .bancho_state_service
                    .send_user_stats_packet(SendUserStatsPacketRequest {
                        user_query: Some(
                            UserQuery::SessionId(session_id.to_owned()).into(),
                        ),
                        to: Some(
                            BanchoPacketTarget::SessionId(session_id).into(),
                        ),
                    })
                    .await?;

                Ok(HandleCompleted {})
            },
        }
    }

    async fn presence_request_all(
        &self,
        request: PresenceRequestAllRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .presence_request_all(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(HandleCompleted {})
            },
        }
    }

    async fn spectate_stop(
        &self,
        request: SpectateStopRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .spectate_stop(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(HandleCompleted {})
            },
        }
    }

    async fn spectate_cant(
        &self,
        request: SpectateCantRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .spectate_cant(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(HandleCompleted {})
            },
        }
    }

    async fn lobby_part(
        &self,
        request: LobbyPartRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .lobby_part(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(HandleCompleted {})
            },
        }
    }

    async fn lobby_join(
        &self,
        request: LobbyJoinRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .lobby_join(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(HandleCompleted {})
            },
        }
    }
}
