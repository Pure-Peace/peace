use super::BanchoStateService;
use crate::bancho_state::{
    BanchoStateError, BanchoStatus, DynBanchoStateBackgroundService,
    DynBanchoStateService, DynUserSessionsService, Session, User,
    UserPlayingStats,
};
use async_trait::async_trait;
use peace_pb::bancho_state_rpc::{
    bancho_state_rpc_client::BanchoStateRpcClient, *,
};
use std::{collections::hash_map::Values, sync::Arc};
use tonic::transport::Channel;

#[derive(Clone)]
pub enum BanchoStateServiceImpl {
    Remote(BanchoStateServiceRemote),
    Local(BanchoStateServiceLocal),
}

impl BanchoStateServiceImpl {
    pub fn into_service(self) -> DynBanchoStateService {
        Arc::new(self) as DynBanchoStateService
    }

    pub fn remote(client: BanchoStateRpcClient<Channel>) -> Self {
        Self::Remote(BanchoStateServiceRemote(client))
    }

    pub fn local(
        user_sessions_service: DynUserSessionsService,
        bancho_state_background_service: DynBanchoStateBackgroundService,
    ) -> Self {
        Self::Local(BanchoStateServiceLocal::new(
            user_sessions_service,
            bancho_state_background_service,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct BanchoStateServiceRemote(BanchoStateRpcClient<Channel>);

impl BanchoStateServiceRemote {
    pub fn new(client: BanchoStateRpcClient<Channel>) -> Self {
        Self(client)
    }

    pub fn client(&self) -> BanchoStateRpcClient<Channel> {
        self.0.clone()
    }
}

#[derive(Clone)]
pub struct BanchoStateServiceLocal {
    user_sessions_service: DynUserSessionsService,
    #[allow(dead_code)]
    bancho_state_background_service: DynBanchoStateBackgroundService,
}

impl BanchoStateServiceLocal {
    pub fn new(
        user_sessions_service: DynUserSessionsService,
        bancho_state_background_service: DynBanchoStateBackgroundService,
    ) -> Self {
        Self { user_sessions_service, bancho_state_background_service }
    }
}

#[async_trait]
impl BanchoStateService for BanchoStateServiceImpl {
    async fn broadcast_bancho_packets(
        &self,
        request: BroadcastBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .broadcast_bancho_packets(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                let packet = Arc::new(request.packets);

                let user_sessions = svc.user_sessions_service.user_sessions();
                let user_sessions = user_sessions.read().await;

                for session in user_sessions.indexed_by_session_id.values() {
                    session.push_packet(packet.clone()).await;
                }

                Ok(ExecSuccess {})
            },
        }
    }

    async fn enqueue_bancho_packets(
        &self,
        request: EnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .enqueue_bancho_packets(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                let EnqueueBanchoPacketsRequest { target, packets } = request;

                let packet = Arc::new(packets);
                let target = Into::<BanchoPacketTarget>::into(
                    target.ok_or(BanchoStateError::BanchoPacketTarget)?,
                );

                if let Ok(user_query) = TryInto::<UserQuery>::try_into(target) {
                    svc.user_sessions_service
                        .get(&user_query)
                        .await
                        .ok_or(BanchoStateError::SessionNotExists)?
                        .push_packet(packet)
                        .await;
                } else {
                    todo!("channel handle")
                }

                Ok(ExecSuccess {})
            },
        }
    }

    async fn batch_enqueue_bancho_packets(
        &self,
        request: BatchEnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .batch_enqueue_bancho_packets(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(_svc) => {
                let batch = request.requests;
                for req in batch {
                    self.enqueue_bancho_packets(req).await?;
                }

                Ok(ExecSuccess {})
            },
        }
    }

    async fn dequeue_bancho_packets(
        &self,
        request: DequeueBanchoPacketsRequest,
    ) -> Result<BanchoPackets, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .dequeue_bancho_packets(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                let target = Into::<BanchoPacketTarget>::into(
                    request
                        .target
                        .ok_or(BanchoStateError::BanchoPacketTarget)?,
                );

                let mut data = Vec::new();

                if let Ok(user_query) = TryInto::<UserQuery>::try_into(target) {
                    while let Some(packet) = svc
                        .user_sessions_service
                        .get(&user_query)
                        .await
                        .ok_or(BanchoStateError::SessionNotExists)?
                        .dequeue_packet(None)
                        .await
                    {
                        data.extend(packet.iter());
                    }
                } else {
                    todo!("channel handle")
                }

                Ok(BanchoPackets { data })
            },
        }
    }

    async fn batch_dequeue_bancho_packets(
        &self,
        request: BatchDequeueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .batch_dequeue_bancho_packets(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(_svc) => unimplemented!(),
        }
    }

    async fn create_user_session(
        &self,
        request: CreateUserSessionRequest,
    ) -> Result<CreateUserSessionResponse, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .create_user_session(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                // Create a new user session using the provided request.
                let session_id = svc
                    .user_sessions_service
                    .create(Session::from_request(request)?)
                    .await;

                // Log the session creation.
                info!(target: "session.create", "Session <{session_id}> created");

                // Return the new session ID in a response.
                Ok(CreateUserSessionResponse { session_id })
            },
        }
    }

    async fn delete_user_session(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .delete_user_session(Into::<RawUserQuery>::into(query))
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                // Delete the session using the query.
                svc.user_sessions_service.delete(&query).await;

                // Log that the session was deleted.
                info!(target: "session.delete", "Session <{query:?}> deleted");

                // Return a success message.
                Ok(ExecSuccess {})
            },
        }
    }

    async fn check_user_session_exists(
        &self,
        query: UserQuery,
    ) -> Result<UserSessionExistsResponse, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .check_user_session_exists(Into::<RawUserQuery>::into(query))
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                // Get session based on the provided query
                let session = svc
                    .user_sessions_service
                    .get(&query)
                    .await
                    .ok_or(BanchoStateError::SessionNotExists)?;

                // Update the user's last active time and retrieve their ID.
                let user_id = {
                    let mut user = session.user.write().await;
                    user.update_active();
                    user.id
                };

                // Return the user ID in a response.
                Ok(UserSessionExistsResponse { user_id })
            },
        }
    }

    async fn get_user_session(
        &self,
        query: UserQuery,
    ) -> Result<GetUserSessionResponse, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .get_user_session(Into::<RawUserQuery>::into(query))
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                // Get session based on the provided query
                let session = svc
                    .user_sessions_service
                    .get(&query)
                    .await
                    .ok_or(BanchoStateError::SessionNotExists)?;

                // Get a read lock on the user session data
                let user = session.user.read().await;

                // Create a response with the user session details
                Ok(GetUserSessionResponse {
                    // Copy the session ID into the response
                    session_id: Some(session.id.to_owned()),
                    // Copy the user ID into the response
                    user_id: Some(user.id),
                    // Copy the username into the response
                    username: Some(user.username.to_owned()),
                    // Copy the Unicode username into the response, if it exists
                    username_unicode: user
                        .username_unicode
                        .as_ref()
                        .map(|s| s.to_owned()),
                })
            },
        }
    }

    async fn get_user_session_with_fields(
        &self,
        request: RawUserQueryWithFields,
    ) -> Result<GetUserSessionResponse, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .get_user_session_with_fields(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                // Extract the query and fields from the request
                let query = request
                    .user_query
                    .ok_or(BanchoStateError::SessionNotExists)?;

                // Get session based on the provided query
                let session = svc
                    .user_sessions_service
                    .get(&query.into())
                    .await
                    .ok_or(BanchoStateError::SessionNotExists)?;

                // Initialize the response and extract the requested fields
                let mut res = GetUserSessionResponse::default();
                let fields = UserSessionFields::from(request.fields);

                // Get user read lock
                let user = session.user.read().await;

                // Set the response fields based on the requested fields
                if fields.intersects(UserSessionFields::SessionId) {
                    res.session_id = Some(session.id.to_owned());
                }

                if fields.intersects(UserSessionFields::UserId) {
                    res.user_id = Some(user.id);
                }

                if fields.intersects(UserSessionFields::Username) {
                    res.username = Some(user.username.to_owned());
                }

                if fields.intersects(UserSessionFields::UsernameUnicode) {
                    res.username_unicode =
                        user.username_unicode.as_ref().map(|s| s.to_owned());
                }

                // Return the response
                Ok(res)
            },
        }
    }

    async fn get_all_sessions(
        &self,
    ) -> Result<GetAllSessionsResponse, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .get_all_sessions(GetAllSessionsRequest {})
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                let user_sessions = svc.user_sessions_service.user_sessions();
                // Get a read lock on the `user_sessions` hash map
                let user_sessions = user_sessions.read().await;

                // Define a helper function to collect data from the hash map
                async fn collect_data<K>(
                    values: Values<'_, K, Arc<Session>>,
                ) -> Vec<UserData> {
                    // Use `join_all` to asynchronously process all elements in
                    // the `values` iterator
                    futures::future::join_all(values.map(|session| async {
                        let User {
                            id: user_id,
                            username,
                            username_unicode,
                            privileges,
                            last_active,
                            ..
                        } = &*session.user.read().await;

                        UserData {
                            session_id: session.id.to_owned(),
                            user_id: *user_id,
                            username: username.to_owned(),
                            username_unicode: username_unicode.to_owned(),
                            privileges: *privileges,
                            connection_info: Some(
                                session.connection_info.to_owned(),
                            ),
                            created_at: session.created_at.to_string(),
                            last_active: last_active.to_string(),
                            queued_packets: session.queued_packets().await
                                as i32,
                        }
                    }))
                    .await
                }

                // Collect session data by index
                let indexed_by_session_id =
                    collect_data(user_sessions.indexed_by_session_id.values())
                        .await;
                let indexed_by_user_id =
                    collect_data(user_sessions.indexed_by_user_id.values())
                        .await;
                let indexed_by_username =
                    collect_data(user_sessions.indexed_by_username.values())
                        .await;
                let indexed_by_username_unicode = collect_data(
                    user_sessions.indexed_by_username_unicode.values(),
                )
                .await;

                // Return a `GetAllSessionsResponse` message containing the
                // session data
                Ok(GetAllSessionsResponse {
                    len: user_sessions.len() as u64,
                    indexed_by_session_id,
                    indexed_by_user_id,
                    indexed_by_username,
                    indexed_by_username_unicode,
                })
            },
        }
    }

    async fn send_user_stats_packet(
        &self,
        request: SendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => svc
                .client()
                .send_user_stats_packet(request)
                .await
                .map_err(BanchoStateError::RpcError)
                .map(|resp| resp.into_inner()),
            BanchoStateServiceImpl::Local(svc) => {
                // Extract the query and fields from the request
                let query = request
                    .user_query
                    .ok_or(BanchoStateError::SessionNotExists)?;

                // Get session based on the provided query
                let session = svc
                    .user_sessions_service
                    .get(&query.into())
                    .await
                    .ok_or(BanchoStateError::SessionNotExists)?;

                let user_stats_packet = {
                    let User {
                        id: user_id,
                        bancho_status:
                            BanchoStatus {
                                online_status,
                                description,
                                beatmap_id,
                                beatmap_md5,
                                mods,
                                mode,
                            },
                        playing_stats:
                            UserPlayingStats {
                                rank,
                                pp_v2,
                                accuracy,
                                total_score,
                                ranked_score,
                                playcount,
                                ..
                            },
                        ..
                    } = &*session.user.read().await;

                    bancho_packets::server::user_stats(
                        *user_id,
                        *online_status as u8,
                        description.to_owned(),
                        beatmap_md5.to_owned(),
                        mods.bits(),
                        *mode as u8,
                        *beatmap_id,
                        *ranked_score,
                        *accuracy,
                        *playcount,
                        *total_score,
                        *rank,
                        *pp_v2 as i16,
                    )
                };

                self.enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
                    target: request.to,
                    packets: user_stats_packet,
                })
                .await?;

                Ok(ExecSuccess {})
            },
        }
    }
}
