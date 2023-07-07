use super::packet_processor::PacketProcessor;
use crate::{
    bancho::{
        traits::*, BanchoServiceError, LoginError, ProcessBanchoPacketError,
    },
    bancho_state::DynBanchoStateService,
    chat::DynChatService,
    gateway::bancho_endpoints::components::BanchoClientToken,
    geoip::DynGeoipService,
    FromRpcClient, IntoService, RpcClient,
};
use bancho_packets::{server, Packet, PacketBuilder, PacketId, PacketReader};
use peace_pb::{
    bancho::{bancho_rpc_client::BanchoRpcClient, *},
    bancho_state::*,
    chat::{CreateQueueRequest, GetPublicChannelsResponse},
    ConvertError,
};
use peace_repositories::users::DynUsersRepository;
use std::{net::IpAddr, str::FromStr, sync::Arc, time::Instant};
use tonic::{async_trait, transport::Channel};
use tools::{lazy_init, tonic_utils::RawRequest, Ulid};

#[derive(Clone)]
pub struct BanchoServiceImpl {
    pub users_repository: DynUsersRepository,
    pub bancho_state_service: DynBanchoStateService,
    pub password_service: DynPasswordService,
    pub bancho_background_service: DynBanchoBackgroundService,
    pub geoip_service: DynGeoipService,
    pub chat_service: DynChatService,
}

impl BanchoServiceImpl {
    #[inline]
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

impl BanchoService for BanchoServiceImpl {}

impl IntoService<DynBanchoService> for BanchoServiceImpl {
    #[inline]
    fn into_service(self) -> DynBanchoService {
        Arc::new(self) as DynBanchoService
    }
}

#[async_trait]
impl Login for BanchoServiceImpl {
    async fn login(
        &self,
        client_ip: IpAddr,
        request: LoginRequest,
    ) -> Result<LoginSuccess, BanchoServiceError> {
        const LOG_TARGET: &str = "bancho_service::login";

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
                let user = self
                    .users_repository
                    .get_user_by_username(
                        Some(username.as_str()),
                        Some(username.as_str()),
                    )
                    .await
                    .map_err(LoginError::UserNotExists)?;

                let () = self
                    .password_service
                    .verify_password(user.password.as_str(), password.as_str())
                    .await
                    .map_err(LoginError::PasswordError)?;

                user
            } else {
                peace_db::peace::entity::users::Model {
                    id: MOCK_COUNT.add(1) as i32,
                    name: username.to_owned(),
                    name_safe: UsernameAscii::from_str(username.as_str())
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
        let user = self
            .users_repository
            .get_user_by_username(
                Some(username.as_str()),
                Some(username.as_str()),
            )
            .await
            .map_err(LoginError::UserNotExists)?;

        #[cfg(not(feature = "bancho-mock-test"))]
        let () = self
            .password_service
            .verify_password(user.password.as_str(), password.as_str())
            .await
            .map_err(LoginError::PasswordError)?;

        let geoip_data =
            self.geoip_service.lookup_with_ip_address(client_ip).await.ok();

        let CreateUserSessionResponse { session_id, signature } = self
            .bancho_state_service
            .create_user_session(CreateUserSessionRequest {
                user_id: user.id,
                username: user.name.to_owned(),
                username_unicode: user.name_unicode.to_owned(),
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

        if let Err(err) = self
            .chat_service
            .create_queue(CreateQueueRequest {
                user_id: user.id,
                username: user.name.to_owned(),
                username_unicode: user.name_unicode,
                privileges: 1,
            })
            .await
        {
            warn!(
                "Failed to create chat packet queue for user {}({}): {}",
                user.id, user.name, err
            )
        }

        let mut packet_builder = PacketBuilder::new()
            .add(server::ProtocolVersion::new(19))
            .add(server::LoginReply::new(bancho_packets::LoginResult::Success(
                user.id,
            )))
            .add(server::BanchoPrivileges::new(1))
            .add(server::SilenceEnd::new(0)) // todo
            .add(server::FriendsList::new(&[])) // todo
            .add(server::Notification::new("welcome to peace!".into()));

        let () = {
            let _ = self
                .chat_service
                .get_public_channels()
                .await
                .map(|GetPublicChannelsResponse { channels }| {
                    for channel in channels {
                        packet_builder.extend(server::ChannelInfo::pack(
                            channel.name.into(),
                            channel
                                .description
                                .map(|s| s.into())
                                .unwrap_or_default(),
                            channel
                                .counter
                                .map(|c| c.bancho as i16)
                                .unwrap_or_default(),
                        ));
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
            signature,
            user_id: user.id,
            packets: packet_builder.build(),
        })
    }
}
#[async_trait]
impl BatchProcessPackets for BanchoServiceImpl {
    async fn batch_process_bancho_packets(
        &self,
        request: BatchProcessBanchoPacketsRequest,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        const LOG_TARGET: &str = "bancho::process_packets";

        let BatchProcessBanchoPacketsRequest { session_id, user_id, packets } =
            request;

        let reader = PacketReader::new(&packets);
        let (mut processed, mut failed) = (0, 0);

        let mut builder = None::<PacketBuilder>;

        for packet in reader {
            info!(target: LOG_TARGET, "Received: {packet}");
            let start = Instant::now();

            match self.process_bancho_packet(&session_id, user_id, packet).await
            {
                Ok(HandleCompleted { packets: Some(packets) }) => {
                    lazy_init!(builder => builder.extend(packets), PacketBuilder::from(packets));
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

            info!(target: LOG_TARGET, " - Processed in: {:?}", start.elapsed());
        }

        if failed == processed {
            return Err(ProcessBanchoPacketError::FailedToProcessAll);
        }

        Ok(HandleCompleted { packets: builder.map(|b| b.build()) })
    }
}

#[async_trait]
impl ProcessPackets for BanchoServiceImpl {
    #[inline]
    async fn process_bancho_packet(
        &self,
        session_id: &str,
        user_id: i32,
        packet: Packet<'_>,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        let processor = PacketProcessor {
            session_id,
            user_id,
            packet,
            bancho_service: self,
            bancho_state_service: self.bancho_state_service.as_ref(),
            chat_service: self.chat_service.as_ref(),
        };

        Ok(match processor.packet.id {
            PacketId::OSU_PING => HandleCompleted::default(),
            // Message
            PacketId::OSU_SEND_PUBLIC_MESSAGE => {
                processor.send_public_message().await?
            },
            PacketId::OSU_SEND_PRIVATE_MESSAGE => {
                processor.send_private_message().await?
            },
            PacketId::OSU_USER_CHANNEL_JOIN => {
                processor.user_channel_join().await?
            },
            PacketId::OSU_USER_CHANNEL_PART => {
                processor.user_channel_part().await?
            },
            // User
            PacketId::OSU_USER_REQUEST_STATUS_UPDATE => {
                processor.user_request_status_update().await?
            },
            PacketId::OSU_USER_PRESENCE_REQUEST_ALL => {
                processor.user_presence_request_all().await?
            },
            PacketId::OSU_USER_STATS_REQUEST => {
                processor.user_stats_request().await?
            },
            PacketId::OSU_USER_CHANGE_ACTION => {
                processor.user_change_action().await?
            },
            PacketId::OSU_USER_RECEIVE_UPDATES => {
                processor.user_receive_updates().await?
            },
            PacketId::OSU_USER_FRIEND_ADD => todo!(),
            PacketId::OSU_USER_FRIEND_REMOVE => todo!(),
            PacketId::OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS => {
                processor.user_toggle_block_non_friend_dms().await?
            },
            PacketId::OSU_USER_LOGOUT => processor.user_logout().await?,
            PacketId::OSU_USER_SET_AWAY_MESSAGE => todo!(),
            PacketId::OSU_USER_PRESENCE_REQUEST => {
                processor.user_presence_request().await?
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
                    processor.packet.id,
                ))
            },
        })
    }
}
#[async_trait]
impl ClientPing for BanchoServiceImpl {
    async fn ping(
        &self,
        PingRequest { user_id, session_id, signature }: PingRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.bancho_state_service
            .check_user_token(BanchoClientToken {
                user_id,
                session_id: Ulid::from_str(session_id.as_str())
                    .map_err(ConvertError::DecodingError)?,
                signature,
            })
            .await?;

        Ok(HandleCompleted::default())
    }
}
#[async_trait]
impl RequestStatusUpdate for BanchoServiceImpl {
    async fn request_status_update(
        &self,
        request: RequestStatusUpdateRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        let RequestStatusUpdateRequest { session_id } = request;

        let user_query = Some(
            UserQuery::SessionId(
                Ulid::from_str(session_id.as_str())
                    .map_err(ConvertError::DecodingError)?,
            )
            .into(),
        );

        let _resp = self
            .bancho_state_service
            .send_user_stats_packet(SendUserStatsPacketRequest {
                user_query: user_query.clone(),
                to: user_query,
            })
            .await?;

        Ok(HandleCompleted::default())
    }
}
#[async_trait]
impl PresenceRequestAll for BanchoServiceImpl {
    async fn presence_request_all(
        &self,
        request: PresenceRequestAllRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        let PresenceRequestAllRequest { session_id } = request;

        let _resp = self
            .bancho_state_service
            .send_all_presences(SendAllPresencesRequest {
                to: Some(
                    UserQuery::SessionId(
                        Ulid::from_str(session_id.as_str())
                            .map_err(ConvertError::DecodingError)?,
                    )
                    .into(),
                ),
            })
            .await?;

        Ok(HandleCompleted::default())
    }
}
#[async_trait]
impl RequestStats for BanchoServiceImpl {
    async fn request_stats(
        &self,
        request: StatsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        let StatsRequest { session_id, request_users } = request;
        let _resp = self
            .bancho_state_service
            .batch_send_user_stats_packet(BatchSendUserStatsPacketRequest {
                user_queries: request_users
                    .into_iter()
                    .map(|user_id| UserQuery::UserId(user_id).into())
                    .collect(),
                to: Some(
                    UserQuery::SessionId(
                        Ulid::from_str(session_id.as_str())
                            .map_err(ConvertError::DecodingError)?,
                    )
                    .into(),
                ),
            })
            .await?;
        Ok(HandleCompleted::default())
    }
}
#[async_trait]
impl ChangeAction for BanchoServiceImpl {
    async fn change_action(
        &self,
        request: ChangeActionRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        let ChangeActionRequest {
            session_id,
            online_status,
            description,
            beatmap_md5,
            mods,
            mode,
            beatmap_id,
        } = request;

        let _resp = self
            .bancho_state_service
            .update_user_bancho_status(UpdateUserBanchoStatusRequest {
                user_query: Some(
                    UserQuery::SessionId(
                        Ulid::from_str(session_id.as_str())
                            .map_err(ConvertError::DecodingError)?,
                    )
                    .into(),
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
    }
}

#[async_trait]
impl ReceiveUpdates for BanchoServiceImpl {
    async fn receive_updates(
        &self,
        request: ReceiveUpdatesRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        let ReceiveUpdatesRequest { session_id, presence_filter } = request;

        self.bancho_state_service
            .update_presence_filter(UpdatePresenceFilterRequest {
                user_query: Some(
                    UserQuery::SessionId(
                        Ulid::from_str(session_id.as_str())
                            .map_err(ConvertError::DecodingError)?,
                    )
                    .into(),
                ),
                presence_filter,
            })
            .await?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl ToggleBlockNonFriendDms for BanchoServiceImpl {
    async fn toggle_block_non_friend_dms(
        &self,
        request: ToggleBlockNonFriendDmsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        println!("Got a request: {:?}", request);

        // todo chat service

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl UserLogout for BanchoServiceImpl {
    async fn user_logout(
        &self,
        request: UserLogoutRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.bancho_state_service
            .delete_user_session(UserQuery::SessionId(
                Ulid::from_str(request.session_id.as_str())
                    .map_err(ConvertError::DecodingError)?,
            ))
            .await?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl RequestPresence for BanchoServiceImpl {
    async fn request_presence(
        &self,
        request: PresenceRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        let PresenceRequest { session_id, request_users } = request;

        self.bancho_state_service
            .batch_send_presences(BatchSendPresencesRequest {
                user_queries: request_users
                    .into_iter()
                    .map(|user_id| UserQuery::UserId(user_id).into())
                    .collect(),
                to: Some(
                    UserQuery::SessionId(
                        Ulid::from_str(session_id.as_str())
                            .map_err(ConvertError::DecodingError)?,
                    )
                    .into(),
                ),
            })
            .await?;

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl SpectateStop for BanchoServiceImpl {
    async fn spectate_stop(
        &self,
        request: SpectateStopRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        println!("Got a request: {:?}", request);

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl SpectateCant for BanchoServiceImpl {
    async fn spectate_cant(
        &self,
        request: SpectateCantRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        println!("Got a request: {:?}", request);

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl LobbyPart for BanchoServiceImpl {
    async fn lobby_part(
        &self,
        request: LobbyPartRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        println!("Got a request: {:?}", request);

        Ok(HandleCompleted::default())
    }
}

#[async_trait]
impl LobbyJoin for BanchoServiceImpl {
    async fn lobby_join(
        &self,
        request: LobbyJoinRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        println!("Got a request: {:?}", request);

        Ok(HandleCompleted::default())
    }
}

#[derive(Clone)]
pub struct BanchoServiceRemote(BanchoRpcClient<Channel>);

impl BanchoService for BanchoServiceRemote {}

impl IntoService<DynBanchoService> for BanchoServiceRemote {
    #[inline]
    fn into_service(self) -> DynBanchoService {
        Arc::new(self) as DynBanchoService
    }
}

impl FromRpcClient for BanchoServiceRemote {
    #[inline]
    fn from_client(client: Self::Client) -> Self {
        Self(client)
    }
}

impl RpcClient for BanchoServiceRemote {
    type Client = BanchoRpcClient<Channel>;

    fn client(&self) -> Self::Client {
        self.0.clone()
    }
}

#[async_trait]
impl Login for BanchoServiceRemote {
    async fn login(
        &self,
        client_ip: IpAddr,
        request: LoginRequest,
    ) -> Result<LoginSuccess, BanchoServiceError> {
        self.client()
            .login(RawRequest::add_client_ip(request, client_ip))
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
#[async_trait]
impl BatchProcessPackets for BanchoServiceRemote {
    async fn batch_process_bancho_packets(
        &self,
        request: BatchProcessBanchoPacketsRequest,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        self.client()
            .batch_process_bancho_packets(request)
            .await
            .map_err(ProcessBanchoPacketError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
#[async_trait]
impl ProcessPackets for BanchoServiceRemote {
    #[inline]
    async fn process_bancho_packet(
        &self,
        session_id: &str,
        user_id: i32,
        packet: Packet<'_>,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError> {
        self.client()
            .process_bancho_packet(ProcessBanchoPacketRequest {
                session_id: session_id.to_owned(),
                user_id,
                packet_id: packet.id as i32,
                payload: packet.payload.map(|p| p.to_vec()),
            })
            .await
            .map_err(ProcessBanchoPacketError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
#[async_trait]
impl ClientPing for BanchoServiceRemote {
    async fn ping(
        &self,
        request: PingRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .ping(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
#[async_trait]
impl RequestStatusUpdate for BanchoServiceRemote {
    async fn request_status_update(
        &self,
        request: RequestStatusUpdateRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .request_status_update(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
#[async_trait]
impl PresenceRequestAll for BanchoServiceRemote {
    async fn presence_request_all(
        &self,
        request: PresenceRequestAllRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .presence_request_all(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
#[async_trait]
impl RequestStats for BanchoServiceRemote {
    async fn request_stats(
        &self,
        request: StatsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .request_stats(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
#[async_trait]
impl ChangeAction for BanchoServiceRemote {
    async fn change_action(
        &self,
        request: ChangeActionRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .change_action(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl ReceiveUpdates for BanchoServiceRemote {
    async fn receive_updates(
        &self,
        request: ReceiveUpdatesRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .receive_updates(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl ToggleBlockNonFriendDms for BanchoServiceRemote {
    async fn toggle_block_non_friend_dms(
        &self,
        request: ToggleBlockNonFriendDmsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .toggle_block_non_friend_dms(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl UserLogout for BanchoServiceRemote {
    async fn user_logout(
        &self,
        request: UserLogoutRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .user_logout(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl RequestPresence for BanchoServiceRemote {
    async fn request_presence(
        &self,
        request: PresenceRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .request_presence(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl SpectateStop for BanchoServiceRemote {
    async fn spectate_stop(
        &self,
        request: SpectateStopRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .spectate_stop(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl SpectateCant for BanchoServiceRemote {
    async fn spectate_cant(
        &self,
        request: SpectateCantRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .spectate_cant(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl LobbyPart for BanchoServiceRemote {
    async fn lobby_part(
        &self,
        request: LobbyPartRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .lobby_part(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl LobbyJoin for BanchoServiceRemote {
    async fn lobby_join(
        &self,
        request: LobbyJoinRequest,
    ) -> Result<HandleCompleted, BanchoServiceError> {
        self.client()
            .lobby_join(request)
            .await
            .map_err(BanchoServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
