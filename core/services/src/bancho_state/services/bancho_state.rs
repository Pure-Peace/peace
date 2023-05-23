use super::{
    traits::*, BanchoStateBackgroundServiceImpl, UserSessionsServiceImpl,
};
use crate::{
    bancho_state::{
        BanchoStateError, CreateSessionError, GameMode, Mods, Packet,
        PresenceFilter, Session, SessionFilter, UserOnlineStatus, UserSessions,
    },
    gateway::bancho_endpoints::components::BanchoClientToken,
    signature::DynSignatureService,
    IntoService,
};
use async_trait::async_trait;
use num_traits::FromPrimitive;
use peace_domain::bancho_state::CreateSessionDto;
use peace_pb::{bancho_state::*, base::ExecSuccess};
use std::sync::Arc;
use tools::{atomic::AtomicValue, message_queue::ReceivedMessages};

#[derive(Clone)]
pub struct BanchoStateServiceImpl {
    pub user_sessions_service: Arc<UserSessionsServiceImpl>,
    pub bancho_state_background_service: Arc<BanchoStateBackgroundServiceImpl>,
    pub signature_service: DynSignatureService,
}

impl BanchoStateServiceImpl {
    #[inline]
    pub fn new(
        user_sessions_service: Arc<UserSessionsServiceImpl>,
        bancho_state_background_service: Arc<BanchoStateBackgroundServiceImpl>,
        signature_service: DynSignatureService,
    ) -> Self {
        Self {
            user_sessions_service,
            bancho_state_background_service,
            signature_service,
        }
    }
}

impl IntoService<DynBanchoStateService> for BanchoStateServiceImpl {
    #[inline]
    fn into_service(self) -> DynBanchoStateService {
        Arc::new(self) as DynBanchoStateService
    }
}

#[async_trait]
impl BanchoStateService for BanchoStateServiceImpl {}

#[async_trait]
impl UpdateUserBanchoStatus for BanchoStateServiceImpl {
    async fn update_user_bancho_status(
        &self,
        request: UpdateUserBanchoStatusRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let UpdateUserBanchoStatusRequest {
            user_query,
            online_status,
            description,
            beatmap_md5,
            mods,
            mode,
            beatmap_id,
        } = request;

        // Extract the query and fields from the request
        let query = user_query.ok_or(BanchoStateError::InvalidArgument)?;

        let session = self
            .user_sessions_service
            .get(&query.into_user_query()?)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        let online_status =
            UserOnlineStatus::from_i32(online_status).unwrap_or_default();
        let mods = Mods::from(mods);
        let mode = GameMode::from_i32(mode).unwrap_or_default();

        session.bancho_status.update_all(
            online_status,
            description,
            beatmap_id as u32,
            beatmap_md5,
            mods,
            mode,
        );

        // todo update stats from database

        self.broadcast_bancho_packets(BroadcastBanchoPacketsRequest {
            packets: session.user_stats_packet(),
        })
        .await?;

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl UpdatePresenceFilter for BanchoStateServiceImpl {
    async fn update_presence_filter(
        &self,
        request: UpdatePresenceFilterRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        // Extract the query and fields from the request
        let query =
            request.user_query.ok_or(BanchoStateError::InvalidArgument)?;

        let presence_filter = PresenceFilter::from_i32(request.presence_filter)
            .ok_or(BanchoStateError::InvalidArgument)?;

        // Get session based on the provided query
        let session = self
            .user_sessions_service
            .get(&query.into_user_query()?)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        session.presence_filter.set(presence_filter.into());

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl BatchSendPresences for BanchoStateServiceImpl {
    async fn batch_send_presences(
        &self,
        request: BatchSendPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        if request.user_queries.is_empty() {
            return Ok(ExecSuccess {})
        }
        let to = request
            .to
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_packet_target()?;

        let presences_packets = {
            let mut presences_packets = Vec::new();

            let indexes =
                self.user_sessions_service.user_sessions().read().await;

            for raw_query in request.user_queries {
                let query = raw_query.into_user_query()?;
                let session = match &query {
                    UserQuery::UserId(user_id) => indexes.user_id.get(user_id),
                    UserQuery::Username(username) =>
                        indexes.username.get(username),
                    UserQuery::UsernameUnicode(username_unicode) =>
                        indexes.username_unicode.get(username_unicode),
                    UserQuery::SessionId(session_id) =>
                        indexes.session_id.get(session_id),
                };

                let session = match session {
                    Some(s) => s,
                    None => continue,
                };

                if SessionFilter::session_is_target(session, &to) {
                    continue
                };

                presences_packets.extend(session.user_presence_packet());
            }

            presences_packets
        };

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            target: Some(to.into()),
            packets: presences_packets,
        })
        .await?;

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl SendAllPresences for BanchoStateServiceImpl {
    async fn send_all_presences(
        &self,
        request: SendAllPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let to = request
            .to
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_packet_target()?;

        let presences_packets = {
            let mut presences_packets = Vec::new();

            let user_sessions =
                self.user_sessions_service.user_sessions().read().await;

            for session in user_sessions.values() {
                if SessionFilter::session_is_target(session, &to) {
                    continue
                };

                presences_packets.extend(session.user_presence_packet());
            }

            presences_packets
        };

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            target: Some(to.into()),
            packets: presences_packets,
        })
        .await?;

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl BatchSendUserStatsPacket for BanchoStateServiceImpl {
    async fn batch_send_user_stats_packet(
        &self,
        request: BatchSendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        if request.user_queries.is_empty() {
            return Ok(ExecSuccess {})
        }
        let to = request
            .to
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_packet_target()?;

        let user_stats_packets = {
            let mut user_stats_packets = Vec::new();

            let indexes =
                self.user_sessions_service.user_sessions().read().await;

            for raw_query in request.user_queries {
                let query = raw_query.into_user_query()?;
                let session = match &query {
                    UserQuery::UserId(user_id) => indexes.user_id.get(user_id),
                    UserQuery::Username(username) =>
                        indexes.username.get(username),
                    UserQuery::UsernameUnicode(username_unicode) =>
                        indexes.username_unicode.get(username_unicode),
                    UserQuery::SessionId(session_id) =>
                        indexes.session_id.get(session_id),
                };

                let session = match session {
                    Some(s) => s,
                    None => continue,
                };

                if SessionFilter::session_is_target(session, &to) {
                    continue
                };

                user_stats_packets.extend(session.user_stats_packet());
            }

            user_stats_packets
        };

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            target: Some(to.into()),
            packets: user_stats_packets,
        })
        .await?;

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl SendUserStatsPacket for BanchoStateServiceImpl {
    async fn send_user_stats_packet(
        &self,
        request: SendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let to = request.to.ok_or(BanchoStateError::InvalidArgument)?;

        // Extract the query and fields from the request
        let query =
            request.user_query.ok_or(BanchoStateError::InvalidArgument)?;

        // Get session based on the provided query
        let session = self
            .user_sessions_service
            .get(&query.into_user_query()?)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            target: Some(to),
            packets: session.user_stats_packet(),
        })
        .await?;

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl GetAllSessions for BanchoStateServiceImpl {
    async fn get_all_sessions(
        &self,
    ) -> Result<GetAllSessionsResponse, BanchoStateError> {
        // Get a read lock on the `user_sessions` hash map
        let user_sessions = self.user_sessions_service.user_sessions();
        let indexes = user_sessions.read().await;

        #[inline]
        fn collect_data<'a, I>(values: I) -> Vec<UserData>
        where
            I: Iterator<Item = &'a Arc<Session>>,
        {
            values
                .map(|session| UserData {
                    json: serde_json::to_string(session)
                        .unwrap_or_else(|err| format!("err: {:?}", err)),
                })
                .collect()
        }

        // Collect session data by index
        let indexed_by_session_id = collect_data(indexes.session_id.values());
        let indexed_by_user_id = collect_data(indexes.user_id.values());
        let indexed_by_username = collect_data(indexes.username.values());
        let indexed_by_username_unicode =
            collect_data(indexes.username_unicode.values());

        // Return a `GetAllSessionsResponse` message containing the
        // session data
        Ok(GetAllSessionsResponse {
            len: user_sessions.len() as u64,
            indexed_by_session_id,
            indexed_by_user_id,
            indexed_by_username,
            indexed_by_username_unicode,
        })
    }
}

#[async_trait]
impl ChannelUpdateNotify for BanchoStateServiceImpl {
    async fn channel_update_notify(
        &self,
        request: ChannelUpdateNotifyRequest,
    ) -> Result<ChannelUpdateNotifyResponse, BanchoStateError> {
        let ChannelUpdateNotifyRequest { notify_targets, channel_info } =
            request;

        let channel_info =
            channel_info.ok_or(BanchoStateError::InvalidArgument)?;

        let packets =
            Packet::new_ptr(bancho_packets::server::ChannelInfo::pack(
                channel_info.name.as_str().into(),
                channel_info.description.map(|s| s.into()).unwrap_or_default(),
                channel_info
                    .counter
                    .ok_or(BanchoStateError::InvalidArgument)?
                    .bancho as i16,
            ));

        match notify_targets {
            Some(notify_targets) => {
                let indexes =
                    self.user_sessions_service.user_sessions().read().await;

                let notify_targets = notify_targets
                    .value
                    .into_iter()
                    .map(|t| t.into_user_query())
                    .filter_map(|q| q.ok())
                    .collect::<Vec<UserQuery>>();

                for user_query in notify_targets {
                    if let Some(session) =
                        UserSessions::get_inner(&indexes, &user_query)
                    {
                        session.push_packet(packets.clone()).await;
                    }
                }
            },
            None => {
                self.user_sessions_service
                    .notify_queue
                    .lock()
                    .await
                    .push(packets.clone(), None);
            },
        }

        Ok(ChannelUpdateNotifyResponse::default())
    }
}

#[async_trait]
impl GetUserSessionWithFields for BanchoStateServiceImpl {
    async fn get_user_session_with_fields(
        &self,
        request: RawUserQueryWithFields,
    ) -> Result<GetUserSessionResponse, BanchoStateError> {
        // Extract the query and fields from the request
        let query =
            request.user_query.ok_or(BanchoStateError::InvalidArgument)?;

        // Get session based on the provided query
        let session = self
            .user_sessions_service
            .get(&query.into_user_query()?)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        // Initialize the response and extract the requested fields
        let mut res = GetUserSessionResponse::default();
        let fields = UserSessionFields::from(request.fields);

        // Set the response fields based on the requested fields
        if fields.intersects(UserSessionFields::SessionId) {
            res.session_id = Some(session.id.to_string());
        }

        if fields.intersects(UserSessionFields::UserId) {
            res.user_id = Some(session.user_id);
        }

        if fields.intersects(UserSessionFields::Username) {
            res.username = Some(session.username.to_string());
        }

        if fields.intersects(UserSessionFields::UsernameUnicode) {
            res.username_unicode =
                session.username_unicode.load().as_ref().map(|s| s.to_string());
        }

        // Return the response
        Ok(res)
    }
}

#[async_trait]
impl GetUserSession for BanchoStateServiceImpl {
    async fn get_user_session(
        &self,
        query: UserQuery,
    ) -> Result<GetUserSessionResponse, BanchoStateError> {
        // Get session based on the provided query
        let session = self
            .user_sessions_service
            .get(&query)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        // Create a response with the user session details
        Ok(GetUserSessionResponse {
            // Copy the session ID into the response
            session_id: Some(session.id.to_string()),
            // Copy the user ID into the response
            user_id: Some(session.user_id),
            // Copy the username into the response
            username: Some(session.username.to_string()),
            // Copy the Unicode username into the response, if it exists
            username_unicode: session
                .username_unicode
                .load()
                .as_ref()
                .map(|s| s.to_string()),
        })
    }
}

#[async_trait]
impl IsUserOnline for BanchoStateServiceImpl {
    async fn is_user_online(
        &self,
        query: UserQuery,
    ) -> Result<UserOnlineResponse, BanchoStateError> {
        let session = self
            .user_sessions_service
            .get(&query)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        Ok(UserOnlineResponse {
            user_id: session.user_id,
            session_id: session.id.to_string(),
        })
    }
}

#[async_trait]
impl CheckUserToken for BanchoStateServiceImpl {
    async fn check_user_token(
        &self,
        token: BanchoClientToken,
    ) -> Result<CheckUserTokenResponse, BanchoStateError> {
        if !self
            .signature_service
            .verify(token.content().into(), token.signature.into())
            .await
            .map_err(|_| BanchoStateError::InvalidToken)?
        {
            return Err(BanchoStateError::InvalidToken)
        }

        let session = self
            .user_sessions_service
            .get(&UserQuery::SessionId(token.session_id))
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        session.update_active();

        Ok(CheckUserTokenResponse::default())
    }
}

#[async_trait]
impl DeleteUserSession for BanchoStateServiceImpl {
    async fn delete_user_session(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.user_sessions_service.delete(&query).await;
        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl CreateUserSession for BanchoStateServiceImpl {
    async fn create_user_session(
        &self,
        request: CreateUserSessionRequest,
    ) -> Result<CreateUserSessionResponse, BanchoStateError> {
        let CreateUserSessionRequest {
            user_id,
            username,
            username_unicode,
            privileges,
            client_version,
            utc_offset,
            display_city,
            only_friend_pm_allowed,
            connection_info,
        } = request;

        // Create a new user session using the provided request.
        let session = self
            .user_sessions_service
            .create(CreateSessionDto {
                user_id,
                username,
                username_unicode,
                privileges,
                client_version,
                utc_offset: utc_offset as u8,
                display_city,
                only_friend_pm_allowed,
                connection_info: connection_info
                    .ok_or(CreateSessionError::InvalidConnectionInfo)?
                    .into(),
                initial_packets: None,
            })
            .await;

        let session_id = session.id.to_string();
        let signature = self
            .signature_service
            .sign(
                BanchoClientToken::encode_content(user_id, &session_id).into(),
            )
            .await?;

        // Return the new session ID in a response.
        Ok(CreateUserSessionResponse { session_id, signature })
    }
}

#[async_trait]
impl DequeueBanchoPackets for BanchoStateServiceImpl {
    async fn dequeue_bancho_packets(
        &self,
        request: DequeueBanchoPacketsRequest,
    ) -> Result<BanchoPackets, BanchoStateError> {
        let target = request
            .target
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_packet_target()?;

        let mut data = Vec::new();

        if let Ok(user_query) = target.into_user_query() {
            let session = self
                .user_sessions_service
                .get(&user_query)
                .await
                .ok_or(BanchoStateError::SessionNotExists)?;

            let mut session_packet_queue = session.packets_queue.lock().await;

            while let Some(packet) =
                session.dequeue_packet(Some(&mut session_packet_queue)).await
            {
                data.extend(packet);
            }

            if let Some(ReceivedMessages { messages, last_msg_id }) = self
                .user_sessions_service
                .notify_queue()
                .lock()
                .await
                .receive(&session.user_id, &session.notify_index.load(), None)
            {
                for packet in messages {
                    data.extend(packet);
                }

                session.notify_index.set(last_msg_id.into());
            }
        } else {
            todo!("channel handle")
        }

        Ok(BanchoPackets { data })
    }
}

#[async_trait]
impl BatchEnqueueBanchoPackets for BanchoStateServiceImpl {
    async fn batch_enqueue_bancho_packets(
        &self,
        request: BatchEnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let BatchEnqueueBanchoPacketsRequest { targets, packets } = request;
        let packets = Packet::new_ptr(packets);

        let user_sessions =
            self.user_sessions_service.user_sessions().read().await;

        for target in targets {
            let target = target.into_packet_target()?;

            if let Ok(user_query) = target.into_user_query() {
                if let Some(session) =
                    UserSessions::get_inner(&user_sessions, &user_query)
                {
                    session.push_packet(packets.clone()).await;
                }
            } else {
                todo!("channel handle")
            }
        }

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl EnqueueBanchoPackets for BanchoStateServiceImpl {
    async fn enqueue_bancho_packets(
        &self,
        request: EnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let EnqueueBanchoPacketsRequest { target, packets } = request;

        let target = target
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_packet_target()?;

        if let Ok(user_query) = TryInto::<UserQuery>::try_into(target) {
            self.user_sessions_service
                .get(&user_query)
                .await
                .ok_or(BanchoStateError::SessionNotExists)?
                .push_packet(packets.into())
                .await;
        } else {
            todo!("channel handle")
        }

        Ok(ExecSuccess {})
    }
}

#[async_trait]
impl BroadcastBanchoPackets for BanchoStateServiceImpl {
    async fn broadcast_bancho_packets(
        &self,
        request: BroadcastBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let packet = Packet::new_ptr(request.packets);

        let user_sessions =
            self.user_sessions_service.user_sessions.read().await;

        for session in user_sessions.values() {
            session.push_packet(packet.clone()).await;
        }

        Ok(ExecSuccess {})
    }
}
