use crate::{
    bancho::{
        packet_processor, BanchoService, BanchoServiceError,
        DynBanchoBackgroundService, DynBanchoService, DynPasswordService,
        LoginError, ProcessBanchoPacketError,
    },
    bancho_state::DynBanchoStateService,
    chat::DynChatService,
    geoip::DynGeoipService,
};
use bancho_packets::{server, Packet, PacketBuilder, PacketId, PacketReader};
use peace_pb::{
    bancho::{bancho_rpc_client::BanchoRpcClient, *},
    bancho_state::*,
    chat::GetPublicChannelsResponse,
};
use peace_repositories::users::DynUsersRepository;
use std::{net::IpAddr, sync::Arc, time::Instant};
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
        chat_service: DynChatService,
    ) -> Self {
        Self::Local(BanchoServiceLocal::new(
            users_repository,
            bancho_state_service,
            password_service,
            bancho_background_service,
            geoip_service,
            chat_service,
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
    pub users_repository: DynUsersRepository,
    pub bancho_state_service: DynBanchoStateService,
    pub password_service: DynPasswordService,
    #[allow(dead_code)]
    pub bancho_background_service: DynBanchoBackgroundService,
    #[allow(dead_code)]
    pub geoip_service: DynGeoipService,
    #[allow(dead_code)]
    pub chat_service: DynChatService,
}

impl BanchoServiceLocal {
    pub fn new(
        users_repository: DynUsersRepository,
        bancho_state_service: DynBanchoStateService,
        password_service: DynPasswordService,
        bancho_background_service: DynBanchoBackgroundService,
        geoip_service: DynGeoipService,
        chat_service: DynChatService,
    ) -> Self {
        Self {
            users_repository,
            bancho_state_service,
            password_service,
            bancho_background_service,
            geoip_service,
            chat_service,
        }
    }
}

pub struct PacketContext<'a> {
    pub session_id: &'a str,
    pub user_id: i32,
    pub packet: Packet<'a>,
    pub svc_local: &'a BanchoServiceLocal,
    pub svc_impl: &'a BanchoServiceImpl,
}

#[async_trait]
impl BanchoService for BanchoServiceImpl {
    async fn login(
        &self,
        client_ip: IpAddr,
        request: LoginRequest,
    ) -> Result<LoginSuccess, BanchoServiceError> {
        const LOG_TARGET: &str = "bancho_service::login";

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
                    ..
                } = request;

                info!(
                    target: LOG_TARGET,
                    "Login request: {username} [{client_version}] ({client_ip})"
                );
                let start = Instant::now();

                // MOCK -------------------
                #[cfg(feature = "bancho-mock-test")]
                let user = {
                    use chrono::Utc;
                    use peace_domain::users::{UsernameAscii, UsernameUnicode};
                    use tools::atomic::{AtomicOperation, U64};

                    static MOCK_COUNT: U64 = U64::new(10000);
                    const EXCLUDE_USERS: [&str; 2] = ["test1", "test"];
                    if EXCLUDE_USERS.contains(&username.as_str()) {
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
                            .verify_password(
                                user.password.as_str(),
                                password.as_str(),
                            )
                            .await
                            .map_err(LoginError::PasswordError)?;

                        user
                    } else {
                        peace_db::peace::entity::users::Model {
                            id: MOCK_COUNT.add(1) as i32,
                            name: username.to_owned(),
                            name_safe: UsernameAscii::from_str(
                                username.as_str(),
                            )
                            .unwrap()
                            .safe_name()
                            .to_string(),
                            name_unicode: Some(username.to_owned()),
                            name_unicode_safe: Some(
                                UsernameUnicode::from_str(username.as_str())
                                    .unwrap()
                                    .safe_name()
                                    .to_string(),
                            ),
                            password: "".into(),
                            email: "".into(),
                            country: Some("".into()),
                            created_at: Utc::now().into(),
                            updated_at: Utc::now().into(),
                        }
                    }
                };

                #[cfg(not(feature = "bancho-mock-test"))]
                let user = svc
                    .users_repository
                    .get_user_by_username(
                        Some(username.as_str()),
                        Some(username.as_str()),
                    )
                    .await
                    .map_err(LoginError::UserNotExists)?;

                #[cfg(not(feature = "bancho-mock-test"))]
                let () = svc
                    .password_service
                    .verify_password(user.password.as_str(), password.as_str())
                    .await
                    .map_err(LoginError::PasswordError)?;

                let geoip_data = svc
                    .geoip_service
                    .lookup_with_ip_address(client_ip)
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

                let mut packet_builder = PacketBuilder::new()
                    .add(server::ProtocolVersion::new(19))
                    .add(server::LoginReply::new(
                        bancho_packets::LoginResult::Success(user.id),
                    ))
                    .add(server::BanchoPrivileges::new(1))
                    .add(server::SilenceEnd::new(0)) // todo
                    .add(server::FriendsList::new(&[])) // todo
                    .add(server::Notification::new("welcome to peace!".into()));

                let () = {
                    let _ = svc
                        .chat_service
                        .get_public_channels()
                        .await
                        .map(|GetPublicChannelsResponse { channels }| {
                            for channel in channels {
                                packet_builder.extend(
                                    server::ChannelInfo::pack(
                                        channel.name.into(),
                                        channel
                                            .description
                                            .map(|s| s.into())
                                            .unwrap_or_default(),
                                        channel
                                            .counter
                                            .map(|c| c.bancho as i16)
                                            .unwrap_or_default(),
                                    ),
                                );
                            }
                        })
                        .map_err(|err| {
                            error!(
                                target: LOG_TARGET,
                                "Failed to fetch channel info, err: {:?}", err
                            );
                            err
                        });

                    packet_builder.extend(server::ChannelInfoEnd::new());
                };

                info!(
                    target: LOG_TARGET,
                    "Logged in: {} [{}] ({}), time spent: {:?}",
                    user.name_safe,
                    user.id,
                    session_id,
                    start.elapsed()
                );

                Ok(LoginSuccess {
                    session_id,
                    user_id: user.id,
                    packets: packet_builder.build(),
                })
            },
        }
    }

    async fn batch_process_bancho_packets(
        &self,
        request: BatchProcessBanchoPacketsRequest,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        const LOG_TARGET: &str = "bancho::process_packets";

        match self {
            Self::Remote(svc) => svc
                .client()
                .batch_process_bancho_packets(request)
                .await
                .map_err(ProcessBanchoPacketError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(_svc) => {
                let BatchProcessBanchoPacketsRequest {
                    session_id,
                    user_id,
                    packets,
                } = request;

                let reader = PacketReader::new(&packets);
                let (mut processed, mut failed) = (0, 0);

                let mut builder = None::<PacketBuilder>;

                for packet in reader {
                    info!(target: LOG_TARGET, "Received: {packet}");
                    let start = Instant::now();

                    match self
                        .process_bancho_packet(&session_id, user_id, packet)
                        .await
                    {
                        Ok(HandleCompleted { packets: Some(packets) }) => {
                            match builder {
                                Some(ref mut builder) => {
                                    builder.extend(packets)
                                },
                                None => {
                                    builder = Some(PacketBuilder::from(packets))
                                },
                            }
                        },
                        Err(err) => {
                            failed += 1;

                            error!(
                                target: LOG_TARGET,
                                "{err:?} (<{user_id}> [{session_id}])"
                            )
                        },
                        _ => {},
                    }

                    processed += 1;

                    info!(
                        target: LOG_TARGET,
                        " - Processed in: {:?}",
                        start.elapsed()
                    );
                }

                if failed == processed {
                    return Err(ProcessBanchoPacketError::FailedToProcessAll);
                }

                Ok(HandleCompleted { packets: builder.map(|b| b.build()) })
            },
        }
    }

    #[inline]
    async fn process_bancho_packet(
        &self,
        session_id: &str,
        user_id: i32,
        packet: Packet<'_>,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .process_bancho_packet(ProcessBanchoPacketRequest {
                    session_id: session_id.to_owned(),
                    user_id,
                    packet_id: packet.id as i32,
                    payload: packet.payload.map(|p| p.to_vec()),
                })
                .await
                .map_err(ProcessBanchoPacketError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let ctx = PacketContext {
                    session_id,
                    user_id,
                    packet,
                    svc_local: svc,
                    svc_impl: self,
                };

                Ok(match ctx.packet.id {
                    PacketId::OSU_PING => HandleCompleted::default(),
                    // Message
                    PacketId::OSU_SEND_PUBLIC_MESSAGE => {
                        packet_processor::send_public_message(ctx).await?
                    },
                    PacketId::OSU_SEND_PRIVATE_MESSAGE => {
                        packet_processor::send_private_message(ctx).await?
                    },
                    PacketId::OSU_USER_CHANNEL_JOIN => {
                        packet_processor::user_channel_join(ctx).await?
                    },
                    PacketId::OSU_USER_CHANNEL_PART => {
                        packet_processor::user_channel_part(ctx).await?
                    },
                    // User
                    PacketId::OSU_USER_REQUEST_STATUS_UPDATE => {
                        packet_processor::user_request_status_update(ctx)
                            .await?
                    },
                    PacketId::OSU_USER_PRESENCE_REQUEST_ALL => {
                        packet_processor::user_presence_request_all(ctx).await?
                    },
                    PacketId::OSU_USER_STATS_REQUEST => {
                        packet_processor::user_stats_request(ctx).await?
                    },
                    PacketId::OSU_USER_CHANGE_ACTION => {
                        packet_processor::user_change_action(ctx).await?
                    },
                    PacketId::OSU_USER_RECEIVE_UPDATES => {
                        packet_processor::user_receive_updates(ctx).await?
                    },
                    PacketId::OSU_USER_FRIEND_ADD => todo!(),
                    PacketId::OSU_USER_FRIEND_REMOVE => todo!(),
                    PacketId::OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS => {
                        packet_processor::user_toggle_block_non_friend_dms(ctx)
                            .await?
                    },
                    PacketId::OSU_USER_LOGOUT => {
                        packet_processor::user_logout(ctx).await?
                    },
                    PacketId::OSU_USER_SET_AWAY_MESSAGE => todo!(),
                    PacketId::OSU_USER_PRESENCE_REQUEST => {
                        packet_processor::user_presence_request(ctx).await?
                    },
                    // Spectate
                    PacketId::OSU_SPECTATE_START => todo!(),
                    PacketId::OSU_SPECTATE_STOP => todo!(),
                    PacketId::OSU_SPECTATE_CANT => todo!(),
                    PacketId::OSU_SPECTATE_FRAMES => todo!(),
                    // Multiplayer
                    PacketId::OSU_USER_PART_LOBBY => todo!(),
                    PacketId::OSU_USER_JOIN_LOBBY => todo!(),
                    PacketId::OSU_USER_PART_MATCH => todo!(),
                    PacketId::OSU_USER_MATCH_READY => todo!(),
                    PacketId::OSU_USER_CREATE_MATCH => todo!(),
                    PacketId::OSU_USER_JOIN_MATCH => todo!(),
                    PacketId::OSU_MATCH_START => todo!(),
                    PacketId::OSU_MATCH_COMPLETE => todo!(),
                    PacketId::OSU_MATCH_LOAD_COMPLETE => todo!(),
                    PacketId::OSU_MATCH_NO_BEATMAP => todo!(),
                    PacketId::OSU_MATCH_NOT_READY => todo!(),
                    PacketId::OSU_MATCH_FAILED => todo!(),
                    PacketId::OSU_MATCH_HAS_BEATMAP => todo!(),
                    PacketId::OSU_MATCH_SKIP_REQUEST => todo!(),
                    PacketId::OSU_MATCH_CHANGE_TEAM => todo!(),
                    PacketId::OSU_MATCH_CHANGE_SLOT => todo!(),
                    PacketId::OSU_MATCH_LOCK => todo!(),
                    PacketId::OSU_MATCH_CHANGE_SETTINGS => todo!(),
                    PacketId::OSU_MATCH_SCORE_UPDATE => todo!(),
                    PacketId::OSU_MATCH_CHANGE_MODS => todo!(),
                    PacketId::OSU_MATCH_TRANSFER_HOST => todo!(),
                    PacketId::OSU_MATCH_INVITE => todo!(),
                    PacketId::OSU_MATCH_CHANGE_PASSWORD => todo!(),
                    // Tournament
                    PacketId::OSU_TOURNAMENT_MATCH_INFO_REQUEST => todo!(),
                    PacketId::OSU_TOURNAMENT_JOIN_MATCH_CHANNEL => todo!(),
                    PacketId::OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL => todo!(),
                    _ => {
                        return Err(ProcessBanchoPacketError::UnhandledPacket(
                            ctx.packet.id,
                        ))
                    },
                })
            },
        }
    }

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

                Ok(HandleCompleted::default())
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

                Ok(HandleCompleted::default())
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

                Ok(HandleCompleted::default())
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
                Ok(HandleCompleted::default())
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
                Ok(HandleCompleted::default())
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

                Ok(HandleCompleted::default())
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

                // todo chat service

                Ok(HandleCompleted::default())
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
            Self::Local(svc) => {
                svc.bancho_state_service
                    .delete_user_session(UserQuery::SessionId(
                        request.session_id,
                    ))
                    .await?;

                Ok(HandleCompleted::default())
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
            Self::Local(svc) => {
                let PresenceRequest { session_id, request_users } = request;

                svc.bancho_state_service
                    .batch_send_presences(BatchSendPresencesRequest {
                        user_queries: request_users
                            .into_iter()
                            .map(|user_id| UserQuery::UserId(user_id).into())
                            .collect(),
                        to: Some(
                            BanchoPacketTarget::SessionId(session_id).into(),
                        ),
                    })
                    .await?;

                Ok(HandleCompleted::default())
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

                Ok(HandleCompleted::default())
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

                Ok(HandleCompleted::default())
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

                Ok(HandleCompleted::default())
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

                Ok(HandleCompleted::default())
            },
        }
    }
}
