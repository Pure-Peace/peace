use super::{BanchoService, DynBanchoService};
use crate::{
    bancho::{
        BanchoServiceError, DynBanchoBackgroundService, DynPasswordService,
        LoginError,
    },
    bancho_state::DynBanchoStateService,
    geoip::DynGeoipService,
};
use bancho_packets::{server, PacketBuilder};
use peace_pb::{
    bancho::{bancho_rpc_client::BanchoRpcClient, *},
    bancho_state::*,
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
        bancho_background_service: DynBanchoBackgroundService,
        geoip_service: DynGeoipService,
    ) -> Self {
        Self::Local(BanchoServiceLocal::new(
            users_repository,
            bancho_state_service,
            password_service,
            bancho_background_service,
            geoip_service,
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
    #[allow(dead_code)]
    bancho_background_service: DynBanchoBackgroundService,
    #[allow(dead_code)]
    geoip_service: DynGeoipService,
}

impl BanchoServiceLocal {
    pub fn new(
        users_repository: DynUsersRepository,
        bancho_state_service: DynBanchoStateService,
        password_service: DynPasswordService,
        bancho_background_service: DynBanchoBackgroundService,
        geoip_service: DynGeoipService,
    ) -> Self {
        Self {
            users_repository,
            bancho_state_service,
            password_service,
            bancho_background_service,
            geoip_service,
        }
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
                let LoginRequest {
                    username,
                    password,
                    client_version,
                    utc_offset,
                    display_city,
                    only_friend_pm_allowed,
                    client_hashes,
                } = request;
                info!("Receive login request: {username} [{client_version}] ({client_ip})");

                let user = svc
                    .users_repository
                    .get_user_by_username(
                        Some(username.as_str()),
                        Some(username.as_str()),
                    )
                    .await
                    .map_err(LoginError::UserNotExists)?;

                let () = svc
                    .password_service
                    .verify_password(user.password.as_str(), password.as_str())
                    .await
                    .map_err(LoginError::PasswordError)?;

                let geoip_data = svc
                    .geoip_service
                    .lookup_with_ip_address(client_ip.clone())
                    .await
                    .ok();

                let CreateUserSessionResponse { session_id } = svc
                    .bancho_state_service
                    .create_user_session(CreateUserSessionRequest {
                        user_id: user.id,
                        username: user.name.to_owned(),
                        username_unicode: user.name_unicode,
                        privileges: 1,
                        client_version,
                        utc_offset,
                        display_city,
                        only_friend_pm_allowed,
                        connection_info: Some(ConnectionInfo {
                            ip: client_ip.to_string(),
                            geoip_data: geoip_data.map(|g| g.into()),
                        }),
                    })
                    .await?;

                let packet_builder = PacketBuilder::new()
                    .add(server::protocol_version(19))
                    .add(server::login_reply(
                        bancho_packets::LoginResult::Success(user.id),
                    ))
                    .add(server::bancho_privileges(1))
                    .add(server::silence_end(0)) // todo
                    .add(server::user_stats(
                        user.id, 0, "", "", 0, 0, 0, 0, 0., 0, 0, 0,
                        0, // todo
                    ))
                    .add(server::user_presence(
                        user.id, user.name, 0, 0, 1, 0., 0., 0, // todo
                    ))
                    .add(server::friends_list(&[])) // todo
                    .add(server::channel_info("peace", "peace", 0))
                    .add(server::channel_info_end())
                    .add(server::notification("welcome to peace!"));

                info!(target: "bancho.login", "user <{}:{}> logged in (session_id: {})", user.name_safe, user.id, session_id);

                Ok(LoginSuccess {
                    session_id,
                    packet: Some(packet_builder.build()),
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
            Self::Local(svc) => {
                let PresenceRequestAllRequest { session_id } = request;

                let _resp = svc
                    .bancho_state_service
                    .send_all_presences(SendAllPresencesRequest {
                        to: Some(
                            BanchoPacketTarget::SessionId(session_id).into(),
                        ),
                    })
                    .await?;

                Ok(HandleCompleted {})
            },
        }
    }

    async fn request_stats(
        &self,
        request: StatsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .request_stats(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let StatsRequest { session_id, request_users } = request;
                let _resp = svc
                    .bancho_state_service
                    .batch_send_user_stats_packet(
                        BatchSendUserStatsPacketRequest {
                            user_queries: request_users
                                .into_iter()
                                .map(|user_id| {
                                    UserQuery::UserId(user_id).into()
                                })
                                .collect(),
                            to: Some(
                                BanchoPacketTarget::SessionId(session_id)
                                    .into(),
                            ),
                        },
                    )
                    .await?;
                Ok(HandleCompleted {})
            },
        }
    }

    async fn change_action(
        &self,
        request: ChangeActionRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .change_action(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let ChangeActionRequest {
                    session_id,
                    online_status,
                    description,
                    beatmap_md5,
                    mods,
                    mode,
                    beatmap_id,
                } = request;

                let _resp = svc
                    .bancho_state_service
                    .update_user_bancho_status(UpdateUserBanchoStatusRequest {
                        user_query: Some(
                            UserQuery::SessionId(session_id).into(),
                        ),
                        online_status,
                        description,
                        beatmap_md5,
                        mods,
                        mode,
                        beatmap_id,
                    })
                    .await?;
                Ok(HandleCompleted {})
            },
        }
    }

    async fn receive_updates(
        &self,
        request: ReceiveUpdatesRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .receive_updates(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let ReceiveUpdatesRequest { session_id, presence_filter } =
                    request;

                svc.bancho_state_service
                    .update_presence_filter(UpdatePresenceFilterRequest {
                        user_query: Some(
                            UserQuery::SessionId(session_id).into(),
                        ),
                        presence_filter,
                    })
                    .await?;

                Ok(HandleCompleted {})
            },
        }
    }

    async fn toggle_block_non_friend_dms(
        &self,
        request: ToggleBlockNonFriendDmsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .toggle_block_non_friend_dms(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(HandleCompleted {})
            },
        }
    }

    async fn user_logout(
        &self,
        request: UserLogoutRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .user_logout(request)
                .await
                .map_err(BanchoServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                println!("Got a request: {:?}", request);

                Ok(HandleCompleted {})
            },
        }
    }

    async fn request_presence(
        &self,
        request: PresenceRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .request_presence(request)
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
