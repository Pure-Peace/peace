use super::traits::*;
use crate::{
    bancho_state::*, gateway::bancho_endpoints::components::BanchoClientToken,
    signature::DynSignatureService, IntoService, ServiceSnapshot,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use infra_packets::Packet;
use infra_users::{CreateSessionDto, SessionFilter};
use num_traits::FromPrimitive;
use peace_domain::bancho::{
    BanchoPrivileges, GameMode, Mods, PresenceFilter, UserOnlineStatus,
};
use peace_pb::{bancho_state::*, base::ExecSuccess};
use peace_snapshot::{
    CreateSnapshot, CreateSnapshotError, LoadSnapshotFrom, SaveSnapshotTo,
    SnapshotConfig, SnapshotExpired, SnapshotTime, SnapshotType,
};
use std::{path::Path, sync::Arc};
use tools::{atomic::AtomicValue, message_queue::ReceivedMessages};

pub struct BanchoStateServiceSnapshotLoader;

impl BanchoStateServiceSnapshotLoader {
    pub async fn load(
        cfg: &CliBanchoStateServiceSnapshotConfigs,
        signature_service: DynSignatureService,
    ) -> BanchoStateServiceImpl {
        if cfg.should_load_snapshot() {
            let snapshot_path = Path::new(cfg.snapshot_path());
            if snapshot_path.is_file() {
                match BanchoStateServiceSnapshot::load_snapshot_from(
                    cfg.snapshot_type(),
                    cfg.snapshot_path(),
                )
                .await
                {
                    Ok(snapshot) => {
                        if !snapshot
                            .snapshot_expired(cfg.snapshot_expired_secs())
                        {
                            info!(
                                "[BanchoStateSnapshot] Load Bancho state service from snapshot files!"
                            );
                            return BanchoStateServiceImpl::from_snapshot(
                                snapshot,
                                signature_service,
                            )
                            .await;
                        }

                        info!("[BanchoStateSnapshot] Snapshot file founded but already expired (create at: {})", snapshot.create_time);
                    },
                    Err(err) => {
                        warn!("[BanchoStateSnapshot] Failed to load snapshot file from path: \"{}\", err: {}", cfg.snapshot_path(), err);
                    },
                }
            } else {
                info!(
                    "[BanchoStateSnapshot] Snapshot file not found, path: \"{}\"",
                    cfg.snapshot_path(),
                );
            }
        }

        BanchoStateServiceImpl::new(
            UserSessionsServiceImpl::new().into_service(),
            signature_service,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanchoStateServiceSnapshot {
    pub user_sessions: Vec<BanchoSessionData>,
    pub notify_queue: Vec<BanchoMessageData>,
    pub create_time: DateTime<Utc>,
}

impl SnapshotTime for BanchoStateServiceSnapshot {
    fn snapshot_time(&self) -> u64 {
        self.create_time.timestamp() as u64
    }
}

#[derive(Clone)]
pub struct BanchoStateServiceImpl {
    pub user_sessions_service: DynUserSessionsService,
    pub signature_service: DynSignatureService,
}

impl BanchoStateServiceImpl {
    #[inline]
    pub fn new(
        user_sessions_service: DynUserSessionsService,
        signature_service: DynSignatureService,
    ) -> Self {
        Self { user_sessions_service, signature_service }
    }

    #[inline]
    pub async fn from_snapshot(
        snapshot: BanchoStateServiceSnapshot,
        signature_service: DynSignatureService,
    ) -> Self {
        let mut session_indexes =
            SessionIndexes::with_capacity(snapshot.user_sessions.len());

        for u in snapshot.user_sessions {
            let session = Arc::new(u.into());
            session_indexes.add_session(session);
        }

        let user_sessions =
            Arc::new(UserSessions::from_indexes(session_indexes));

        let notify_queue =
            Arc::new(BanchoMessageQueue::from(snapshot.notify_queue));

        let user_sessions_service =
            UserSessionsServiceImpl { user_sessions, notify_queue }
                .into_service();

        Self { user_sessions_service, signature_service }
    }
}

impl IntoService<DynBanchoStateService> for BanchoStateServiceImpl {
    #[inline]
    fn into_service(self) -> DynBanchoStateService {
        Arc::new(self) as DynBanchoStateService
    }
}

#[async_trait]
impl ServiceSnapshot for BanchoStateServiceImpl {
    async fn save_service_snapshot(
        &self,
        snapshot_type: SnapshotType,
        snapshot_path: &str,
    ) -> Result<(), CreateSnapshotError> {
        info!(
            "Saving Bancho state snapshot file to path: \"{}\"...",
            snapshot_path
        );
        let size = self
            .save_snapshot_to(snapshot_type, snapshot_path)
            .await
            .map_err(|err| {
                warn!(
                    "[Failed] Failed to create Bancho state snapshot, err: {err}"
                );
                err
            })?;
        info!("[Success] Bancho state snapshot saved, size: {}", size);

        Ok(())
    }
}

#[async_trait]
impl CreateSnapshot<BanchoStateServiceSnapshot> for BanchoStateServiceImpl {
    async fn create_snapshot(&self) -> BanchoStateServiceSnapshot {
        BanchoStateServiceSnapshot {
            user_sessions: self
                .user_sessions_service
                .user_sessions()
                .create_snapshot()
                .await,
            notify_queue: self
                .user_sessions_service
                .notify_queue()
                .snapshot_messages()
                .await,
            create_time: Utc::now(),
        }
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

        session.extends.bancho_status.update_all(
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

        Ok(ExecSuccess::default())
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

        session.extends.presence_filter.set(presence_filter.into());

        Ok(ExecSuccess::default())
    }
}

#[async_trait]
impl BatchSendPresences for BanchoStateServiceImpl {
    async fn batch_send_presences(
        &self,
        request: BatchSendPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        if request.user_queries.is_empty() {
            return Ok(ExecSuccess::default());
        }
        let to = request
            .to
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_user_query()?;

        let presences_packets = {
            let mut presences_packets = Vec::new();

            let indexes =
                self.user_sessions_service.user_sessions().read().await;

            for raw_query in request.user_queries {
                let query = raw_query.into_user_query()?;
                let session = match &query {
                    UserQuery::UserId(user_id) => indexes.user_id.get(user_id),
                    UserQuery::Username(username) => {
                        indexes.username.get(username)
                    },
                    UserQuery::UsernameUnicode(username_unicode) => {
                        indexes.username_unicode.get(username_unicode)
                    },
                    UserQuery::SessionId(session_id) => {
                        indexes.session_id.get(session_id)
                    },
                };

                let session = match session {
                    Some(s) => s,
                    None => continue,
                };

                if SessionFilter::session_is_target(session, &to) {
                    continue;
                };

                presences_packets.extend(session.user_presence_packet());
            }

            presences_packets
        };

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            user_query: Some(to.into()),
            packets: presences_packets,
        })
        .await?;

        Ok(ExecSuccess::default())
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
            .into_user_query()?;

        let presences_packets = {
            let mut presences_packets = Vec::new();

            let user_sessions =
                self.user_sessions_service.user_sessions().read().await;

            for session in user_sessions.values() {
                if SessionFilter::session_is_target(session, &to) {
                    continue;
                };

                presences_packets.extend(session.user_presence_packet());
            }

            presences_packets
        };

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            user_query: Some(to.into()),
            packets: presences_packets,
        })
        .await?;

        Ok(ExecSuccess::default())
    }
}

#[async_trait]
impl BatchSendUserStatsPacket for BanchoStateServiceImpl {
    async fn batch_send_user_stats_packet(
        &self,
        request: BatchSendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        if request.user_queries.is_empty() {
            return Ok(ExecSuccess::default());
        }
        let to = request
            .to
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_user_query()?;

        let user_stats_packets = {
            let mut user_stats_packets = Vec::new();

            let indexes =
                self.user_sessions_service.user_sessions().read().await;

            for raw_query in request.user_queries {
                let query = raw_query.into_user_query()?;
                let session = match &query {
                    UserQuery::UserId(user_id) => indexes.user_id.get(user_id),
                    UserQuery::Username(username) => {
                        indexes.username.get(username)
                    },
                    UserQuery::UsernameUnicode(username_unicode) => {
                        indexes.username_unicode.get(username_unicode)
                    },
                    UserQuery::SessionId(session_id) => {
                        indexes.session_id.get(session_id)
                    },
                };

                let session = match session {
                    Some(s) => s,
                    None => continue,
                };

                if SessionFilter::session_is_target(session, &to) {
                    continue;
                };

                user_stats_packets.extend(session.user_stats_packet());
            }

            user_stats_packets
        };

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            user_query: Some(to.into()),
            packets: user_stats_packets,
        })
        .await?;

        Ok(ExecSuccess::default())
    }
}

#[async_trait]
impl SendUserStatsPacket for BanchoStateServiceImpl {
    async fn send_user_stats_packet(
        &self,
        request: SendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let to = request.to.ok_or(BanchoStateError::InvalidArgument)?;
        let query =
            request.user_query.ok_or(BanchoStateError::InvalidArgument)?;

        // Get session based on the provided query
        let session = self
            .user_sessions_service
            .get(&query.into_user_query()?)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
            user_query: Some(to),
            packets: session.user_stats_packet(),
        })
        .await?;

        Ok(ExecSuccess::default())
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
            I: Iterator<Item = &'a Arc<BanchoSession>>,
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
            len: user_sessions.length() as u64,
            indexed_by_session_id,
            indexed_by_user_id,
            indexed_by_username,
            indexed_by_username_unicode,
        })
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
            .await?
        {
            return Err(BanchoStateError::SessionNotExists);
        }

        let session = self
            .user_sessions_service
            .get(&UserQuery::SessionId(token.session_id))
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        session.update_active();

        Ok(CheckUserTokenResponse { is_valid: true })
    }
}

#[async_trait]
impl DeleteUserSession for BanchoStateServiceImpl {
    async fn delete_user_session(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.user_sessions_service.delete(&query).await;
        Ok(ExecSuccess::default())
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
            bancho_privileges,
            connection_info,
            country_code,
        } = request;

        let connection_info = connection_info
            .ok_or(CreateSessionError::InvalidConnectionInfo)?
            .into();

        // Create a new user session using the provided request.
        let session = self
            .user_sessions_service
            .create(CreateSessionDto {
                user_id,
                username,
                username_unicode,
                privileges,
                extends: BanchoExtend::new(
                    None,
                    client_version,
                    utc_offset as u8,
                    display_city,
                    only_friend_pm_allowed,
                    BanchoPrivileges::from(bancho_privileges),
                    connection_info,
                    country_code as u8,
                ),
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
        let user_query = request
            .user_query
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_user_query()?;

        let mut data = Vec::new();

        let session = self
            .user_sessions_service
            .get(&user_query)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?;

        data.extend(
            session.extends.packets_queue.dequeue_all_packets(None).await,
        );

        if let Some(ReceivedMessages { messages, last_msg_id }) = self
            .user_sessions_service
            .notify_queue()
            .read()
            .await
            .receive_messages(
                &session.user_id,
                &session.extends.notify_index.load(),
                None,
            )
            .await
        {
            for packet in messages {
                data.extend(packet);
            }

            session.extends.notify_index.set(last_msg_id.into());
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
        let BatchEnqueueBanchoPacketsRequest { user_queries, packets } =
            request;
        let packets = Packet::new_ptr(packets);

        let user_sessions =
            self.user_sessions_service.user_sessions().read().await;

        for user_query in user_queries {
            if let Some(session) = UserSessions::get_inner(
                &user_sessions,
                &user_query.into_user_query()?,
            ) {
                session
                    .extends
                    .packets_queue
                    .push_packet(packets.clone())
                    .await;
            }
        }

        Ok(ExecSuccess::default())
    }
}

#[async_trait]
impl EnqueueBanchoPackets for BanchoStateServiceImpl {
    async fn enqueue_bancho_packets(
        &self,
        request: EnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let EnqueueBanchoPacketsRequest { user_query, packets } = request;

        let user_query = user_query
            .ok_or(BanchoStateError::InvalidArgument)?
            .into_user_query()?;

        self.user_sessions_service
            .get(&user_query)
            .await
            .ok_or(BanchoStateError::SessionNotExists)?
            .extends
            .packets_queue
            .push_packet(packets.into())
            .await;

        Ok(ExecSuccess::default())
    }
}

#[async_trait]
impl BroadcastBanchoPackets for BanchoStateServiceImpl {
    async fn broadcast_bancho_packets(
        &self,
        request: BroadcastBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        let packet = Packet::new_ptr(request.packets);

        self.user_sessions_service
            .notify_queue()
            .write()
            .await
            .push_message(packet, None);

        Ok(ExecSuccess::default())
    }
}
