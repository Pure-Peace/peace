use crate::{
    bancho_state::{BanchoMessageData, BanchoMessageQueue},
    chat::*,
    FromRpcClient, IntoService, RpcClient, ServiceSnapshot,
};
use async_trait::async_trait;
use bancho_packets::server;
use chat::traits::{ChatService, DynChatService};
use chrono::{DateTime, Utc};
use infra_packets::{Packet, PacketsQueue};
use infra_users::CreateSessionDto;
use peace_domain::chat::{ChannelType, Platform};
use peace_message_queue::ReceivedMessages;
use peace_pb::{
    bancho_state::{BanchoPackets, RawUserQuery, UserQuery},
    base::ExecSuccess,
    chat::{
        chat_rpc_client::ChatRpcClient, ChannelInfo, ChatMessageTarget,
        GetPublicChannelsRequest, GetPublicChannelsResponse,
        JoinChannelRequest, LeaveChannelRequest, LoadPublicChannelsRequest,
        LoginRequest, LogoutRequest, SendMessageRequest, SendMessageResponse,
    },
};
use peace_repositories::users::DynUsersRepository;
use peace_snapshot::{
    CreateSnapshot, CreateSnapshotError, LoadSnapshotFrom, SaveSnapshotTo,
    SnapshotConfig, SnapshotExpired, SnapshotTime, SnapshotType,
};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    path::Path,
    sync::Arc,
};
use tokio::sync::RwLock;
use tonic::{transport::Channel as RpcChannel, IntoRequest};
use tools::atomic::{AtomicValue, U32};

#[derive(Clone)]
pub struct ChatServiceImpl {
    pub user_sessions: Arc<UserSessions>,
    pub notify_queue: Arc<BanchoMessageQueue>,
    pub channels: Arc<Channels>,
    pub users_repository: DynUsersRepository,
}

impl ChatServiceImpl {
    #[inline]
    pub fn new(users_repository: DynUsersRepository) -> Self {
        Self {
            user_sessions: UserSessions::default().into(),
            notify_queue: Arc::new(BanchoMessageQueue::default()),
            channels: Channels::default().into(),
            users_repository,
        }
    }

    #[inline]
    pub async fn from_snapshot(
        snapshot: ChatServiceSnapshot,
        users_repository: DynUsersRepository,
    ) -> Self {
        let mut session_indexes =
            SessionIndexes::with_capacity(snapshot.user_sessions.len());

        for u in snapshot.user_sessions {
            let session = Arc::new(u.into());
            session_indexes.add_session(session);
        }

        let notify_queue =
            Arc::new(BanchoMessageQueue::from(snapshot.notify_queue));

        let mut channel_indexes =
            ChannelIndexes::with_capacity(snapshot.channels.len());

        for ch in snapshot.channels {
            let user_count = U32::new(ch.users.len() as u32);
            let users = Arc::new(RwLock::new(HashMap::from_iter(
                ch.users.into_iter().map(|user_id| {
                    (user_id, session_indexes.get(&user_id).map(Arc::downgrade))
                }),
            )));

            let channel = Arc::new(Channel {
                id: ch.id,
                name: ch.name.into(),
                channel_type: ch.channel_type,
                description: ch.description.into(),
                users,
                user_count,
                min_msg_index: ch.min_msg_index.into(),
                message_queue: Arc::new(ch.message_queue.into()),
                created_at: ch.created_at,
            });

            // upgrade user's JoinedChannel to real channel's weak ptr
            for user_id in channel.users.read().await.keys() {
                if let Some(session) = session_indexes.get(user_id) {
                    let mut joined_channels =
                        session.extends.joined_channels.write().await;

                    joined_channels
                        .entry(channel.id)
                        .and_modify(|j| {
                            j.ptr.set(Arc::downgrade(&channel).into())
                        })
                        .or_insert_with(|| {
                            Arc::new(JoinedChannel::from(Arc::downgrade(
                                &channel,
                            )))
                        });
                }
            }

            // Publish channel info update
            notify_queue
                .push_message(Packet::Ptr(channel.info_packets().into()), None)
                .await;

            channel_indexes.add_channel(channel);
        }

        let channels = Arc::new(Channels::from_indexes(channel_indexes));

        let user_sessions =
            Arc::new(UserSessions::from_indexes(session_indexes));

        Self { user_sessions, notify_queue, channels, users_repository }
    }

    #[inline]
    pub fn into_service(self) -> DynChatService {
        Arc::new(self) as DynChatService
    }

    #[inline]
    pub async fn login_inner(
        &self,
        user_id: i32,
        username: String,
        username_unicode: Option<String>,
        privileges: i32,
        platforms: Platform,
    ) -> Result<Arc<ChatSession>, ChatError> {
        let bancho_chat_ext = if platforms.contains(Platform::Bancho) {
            // prepare bancho packets
            let mut channel_packets = VecDeque::new();

            for channel in self.channels.read().await.public_channels.values() {
                channel_packets.push_back(
                    server::ChannelInfo::pack(
                        channel.name.load().as_ref().into(),
                        channel
                            .description
                            .load()
                            .as_deref()
                            .map(|s| s.into())
                            .unwrap_or_default(),
                        channel.user_count.val() as i16,
                    )
                    .into(),
                );
            }

            channel_packets.push_back(server::ChannelInfoEnd::pack().into());

            Some(PacketsQueue::new(channel_packets).into())
        } else {
            None
        };

        let extends = ChatSessionExtend::new(platforms, bancho_chat_ext, None);

        let session = ChatSession::new(CreateSessionDto {
            user_id,
            username,
            username_unicode,
            privileges,
            extends,
        });

        let session = self.user_sessions.create(session.into()).await;

        Ok(session)
    }

    pub async fn get_session(
        &self,
        query: &UserQuery,
        create_if_not_exists: Option<Platform>,
    ) -> Result<Arc<ChatSession>, ChatError> {
        match self.user_sessions.get(query).await {
            Some(session) => {
                session.update_active();
                Ok(session)
            },
            None => {
                if let Some(platforms) = create_if_not_exists {
                    let user = match query {
                        UserQuery::SessionId(_) => {
                            return Err(ChatError::InvalidArgument)
                        },
                        UserQuery::UserId(user_id) => {
                            self.users_repository.get_user_by_id(*user_id).await
                        },
                        UserQuery::Username(username) => {
                            self.users_repository
                                .get_user_by_username(username.as_str())
                                .await
                        },
                        UserQuery::UsernameUnicode(username_unicode) => {
                            self.users_repository
                                .get_user_by_username_unicode(
                                    username_unicode.as_str(),
                                )
                                .await
                        },
                    }?;

                    self.login_inner(
                        user.id,
                        user.name,
                        user.name_unicode,
                        1, // todo
                        platforms,
                    )
                    .await
                } else {
                    Err(ChatError::SessionNotExists)
                }
            },
        }
    }
}

pub struct ChatServiceSnapshotLoader;

impl ChatServiceSnapshotLoader {
    pub async fn load(
        cfg: &CliChatServiceSnapshotConfigs,
        users_repository: DynUsersRepository,
    ) -> ChatServiceImpl {
        if cfg.should_load_snapshot() {
            let snapshot_path = Path::new(cfg.snapshot_path());
            if snapshot_path.is_file() {
                match ChatServiceSnapshot::load_snapshot_from(
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
                                "[ChatSnapshot] Load chat service from snapshot files!"
                            );
                            return ChatServiceImpl::from_snapshot(
                                snapshot,
                                users_repository,
                            )
                            .await;
                        }

                        info!("[ChatSnapshot] Snapshot file founded but already expired (create at: {})", snapshot.create_time);
                    },
                    Err(err) => {
                        warn!("[ChatSnapshot] Failed to load snapshot file from path: \"{}\", err: {}", cfg.snapshot_path(), err);
                    },
                }
            } else {
                info!(
                    "[ChatSnapshot] Snapshot file not found, path: \"{}\"",
                    cfg.snapshot_path(),
                );
            }
        }

        ChatServiceImpl::new(users_repository)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatServiceSnapshot {
    pub user_sessions: Vec<ChatSessionData>,
    pub notify_queue: Vec<BanchoMessageData>,
    pub channels: Vec<ChannelData>,
    pub create_time: DateTime<Utc>,
}

impl SnapshotTime for ChatServiceSnapshot {
    fn snapshot_time(&self) -> u64 {
        self.create_time.timestamp() as u64
    }
}

#[async_trait]
impl CreateSnapshot<ChatServiceSnapshot> for ChatServiceImpl {
    async fn create_snapshot(&self) -> ChatServiceSnapshot {
        ChatServiceSnapshot {
            user_sessions: self.user_sessions.create_snapshot().await,
            notify_queue: self.notify_queue.snapshot_messages().await,
            channels: self.channels.snapshot_channels().await,
            create_time: Utc::now(),
        }
    }
}

impl UserSessionsStore for ChatServiceImpl {
    #[inline]
    fn user_sessions(&self) -> &Arc<UserSessions> {
        &self.user_sessions
    }
}

impl NotifyMessagesQueue for ChatServiceImpl {
    #[inline]
    fn notify_queue(&self) -> &Arc<BanchoMessageQueue> {
        &self.notify_queue
    }
}

impl ChannelStore for ChatServiceImpl {
    #[inline]
    fn channels(&self) -> &Arc<Channels> {
        &self.channels
    }
}

#[async_trait]
impl ServiceSnapshot for ChatServiceImpl {
    async fn save_service_snapshot(
        &self,
        snapshot_type: SnapshotType,
        snapshot_path: &str,
    ) -> Result<(), CreateSnapshotError> {
        info!("Saving chat snapshot file to path: \"{}\"...", snapshot_path);
        let size = self
            .save_snapshot_to(snapshot_type, snapshot_path)
            .await
            .map_err(|err| {
                warn!("[Failed] Failed to create Chat snapshot, err: {err}");
                err
            })?;
        info!("[Success] Chat snapshot saved, size: {}", size);

        Ok(())
    }
}

#[async_trait]
impl ChatService for ChatServiceImpl {
    async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<ExecSuccess, ChatError> {
        const LOG_TARGET: &str = "chat::login";

        let LoginRequest {
            user_id,
            username,
            username_unicode,
            privileges,
            platforms,
        } = request;

        let platforms = Platform::from(platforms);

        let session = self
            .login_inner(
                user_id,
                username,
                username_unicode,
                privileges,
                platforms,
            )
            .await?;

        info!(
            target: LOG_TARGET,
            "User {}({}) logged in",
            session.username.load(),
            session.user_id,
        );

        Ok(ExecSuccess::default())
    }

    async fn logout(
        &self,
        query: UserQuery,
        remove_platforms: Platform,
    ) -> Result<ExecSuccess, ChatError> {
        const LOG_TARGET: &str = "chat::logout";

        let session = match self.user_sessions.get(&query).await {
            Some(session) => session,
            None => return Ok(ExecSuccess::default()),
        };

        let curr_platforms = session.extends.platforms.val();

        // Logout from bancho
        if curr_platforms.contains(Platform::Bancho)
            && remove_platforms.contains(Platform::Bancho)
        {
            session.extends.bancho_ext.set(None);
        }

        // TODO: part from other platforms
        if curr_platforms.contains(Platform::Lazer)
            && remove_platforms.contains(Platform::Lazer)
        {
            todo!("Logout from Lazer")
        }

        if curr_platforms.contains(Platform::Web)
            && remove_platforms.contains(Platform::Web)
        {
            todo!("Logout from Web")
        }

        // do remove platforms
        let platforms = curr_platforms.and(remove_platforms.not());

        if platforms.is_none() {
            // leave all channels
            for channel in session.extends.joined_channels.read().await.values()
            {
                if let Some(channel) = channel.ptr.load().upgrade() {
                    // remove user from channel
                    Channel::remove(&session, &channel).await;

                    // update channel info
                    self.notify_queue.write().await.push_message(
                        Packet::Ptr(channel.info_packets().into()),
                        None,
                    );
                }
            }

            // delete user session
            self.user_sessions.delete(&query).await;

            info!(
                target: LOG_TARGET,
                "User {}({}) logged out",
                session.username.load(),
                session.user_id,
            );

            return Ok(ExecSuccess::default());
        }

        session.extends.platforms.set(platforms.into());

        info!(
            target: LOG_TARGET,
            "User {}({}) leaved from platforms: {:?} ",
            session.username.load(),
            session.user_id,
            remove_platforms.platforms_array()
        );

        Ok(ExecSuccess::default())
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatError> {
        const LOG_TARGET: &str = "chat::send_message";

        let SendMessageRequest { sender, message, target } = request;

        let sender_query =
            sender.ok_or(ChatError::InvalidArgument)?.into_user_query()?;

        let target =
            target.ok_or(ChatError::InvalidArgument)?.into_message_target()?;

        let sender =
            self.get_session(&sender_query, Some(Platform::all())).await?;

        match target {
            ChatMessageTarget::Channel(channel_query) => {
                // get channel
                let channel =
                    match self.channels.get_channel(&channel_query).await {
                        Some(channel) => channel,
                        None => {
                            todo!("channel not exists")
                        },
                    };

                let message_packet = server::SendMessage::pack(
                    sender.username.load().as_ref().into(),
                    Cow::Borrowed(message.as_ref()),
                    channel.name.load().as_ref().into(),
                    sender.user_id,
                )
                .into();

                // push msg into channel packets queue
                channel.message_queue.write().await.push_message_excludes(
                    Packet::Ptr(message_packet),
                    [sender.user_id],
                    None,
                );

                info!(
                    target: LOG_TARGET,
                    "{}({}) @ {}({}): {}",
                    sender.username.load(),
                    sender.user_id,
                    channel.name.load(),
                    channel.id,
                    message
                );
            },
            ChatMessageTarget::User(target_query) => {
                // get target user session
                match self.get_session(&target_query, None).await.ok() {
                    Some(target_user) => {
                        // push msg packet if target user's bancho packets queue is exists
                        if let Some(bancho_ext) =
                            target_user.extends.bancho_ext.load().as_ref()
                        {
                            bancho_ext
                                .packets_queue
                                .push_packet(
                                    server::SendMessage::pack(
                                        sender.username.load().as_ref().into(),
                                        Cow::Borrowed(message.as_ref()),
                                        target_user
                                            .username
                                            .load()
                                            .as_ref()
                                            .into(),
                                        sender.user_id,
                                    )
                                    .into(),
                                )
                                .await;
                        }

                        info!(
                            target: LOG_TARGET,
                            "{}({}) @ {}({}): {}",
                            sender.username.load(),
                            sender.user_id,
                            target_user.username.load(),
                            target_user.user_id,
                            message
                        );
                    },
                    None => {
                        todo!("offline msg handle")
                    },
                };
            },
        }

        Ok(SendMessageResponse::default())
    }

    async fn join_channel(
        &self,
        request: JoinChannelRequest,
    ) -> Result<ExecSuccess, ChatError> {
        let JoinChannelRequest { channel_query, user_query } = request;

        let user_query =
            user_query.ok_or(ChatError::InvalidArgument)?.into_user_query()?;

        let channel_query = channel_query
            .ok_or(ChatError::InvalidArgument)?
            .into_channel_query()?;

        let session =
            self.get_session(&user_query, Some(Platform::all())).await?;

        let channel = match self.channels.get_channel(&channel_query).await {
            Some(channel) => channel,
            None => {
                todo!("channel not exists")
            },
        };

        // add user into channel
        Channel::join(&session, &channel).await;

        // update channel info
        self.notify_queue
            .write()
            .await
            .push_message(Packet::Ptr(channel.info_packets().into()), None);

        Ok(ExecSuccess::default())
    }

    async fn leave_channel(
        &self,
        request: LeaveChannelRequest,
    ) -> Result<ExecSuccess, ChatError> {
        let LeaveChannelRequest { channel_query, user_query } = request;

        let user_query =
            user_query.ok_or(ChatError::InvalidArgument)?.into_user_query()?;

        let channel_query = channel_query
            .ok_or(ChatError::InvalidArgument)?
            .into_channel_query()?;

        let session =
            self.get_session(&user_query, Some(Platform::all())).await?;

        let channel = match self.channels.get_channel(&channel_query).await {
            Some(channel) => channel,
            None => {
                todo!("channel not exists")
            },
        };

        // remove user from channel
        Channel::remove(&session, &channel).await;

        // update channel info
        self.notify_queue
            .write()
            .await
            .push_message(Packet::Ptr(channel.info_packets().into()), None);

        Ok(ExecSuccess::default())
    }

    async fn dequeue_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<BanchoPackets, ChatError> {
        let session = self.get_session(&query, Some(Platform::Bancho)).await?;

        let bancho_ext = match session.extends.bancho_ext.load_full() {
            Some(bancho_ext) => bancho_ext,
            None => todo!("invalid call"),
        };

        let mut data = Vec::new();

        // receive global notify from queue
        if let Some(ReceivedMessages { messages, last_msg_id }) = self
            .notify_queue
            .read()
            .await
            .receive_messages(
                &session.user_id,
                &bancho_ext.notify_index.load(),
                None,
            )
            .await
        {
            for packet in messages {
                data.extend(packet);
            }

            bancho_ext.notify_index.set(last_msg_id.into());
        }

        // get user's joined channels
        let joined_channels = session
            .extends
            .joined_channels
            .read()
            .await
            .iter()
            .map(|(channel_id, channel)| (*channel_id, channel.clone()))
            .collect::<Vec<(u64, Arc<JoinedChannel>)>>();

        let mut invalid_channels = Vec::new();

        // receive msg from each channels, and mark invalid channels ptr
        for (channel_id, joined_channel) in joined_channels {
            match joined_channel.ptr.load().upgrade() {
                Some(channel) => {
                    if let Some(ReceivedMessages { messages, last_msg_id }) =
                        channel
                            .message_queue
                            .read()
                            .await
                            .receive_messages(
                                &session.user_id,
                                &joined_channel.message_index.load(),
                                None,
                            )
                            .await
                    {
                        for packet in messages {
                            data.extend(packet);
                        }

                        match channel.min_msg_index.load().as_deref() {
                            Some(prev_channel_min_msg_id) => {
                                if &last_msg_id < prev_channel_min_msg_id {
                                    channel
                                        .min_msg_index
                                        .set(Some(last_msg_id.into()))
                                }
                            },
                            None => channel
                                .min_msg_index
                                .set(Some(last_msg_id.into())),
                        };

                        joined_channel.message_index.set(last_msg_id.into());
                    }
                },
                None => invalid_channels.push(channel_id),
            }
        }

        // remove invalid channels
        if !invalid_channels.is_empty() {
            let mut joined_channels =
                session.extends.joined_channels.write().await;

            for channel_id in invalid_channels {
                joined_channels.remove(&channel_id);
            }
        }

        // receive msg from session queue
        data.extend(bancho_ext.packets_queue.dequeue_all_packets(None).await);

        Ok(BanchoPackets { data })
    }

    async fn load_public_channels(&self) -> Result<ExecSuccess, ChatError> {
        const LOG_TARGET: &str = "chat::channel::initialize_public_channels";

        // todo: load public channels from database
        let public_channels = vec![
            Channel::new(
                0,
                "#osu".to_string(),
                ChannelType::Public,
                Some("default channel".to_string()),
                None,
            ),
            Channel::new(
                1,
                "#peace".to_string(),
                ChannelType::Public,
                Some("peace channel".to_string()),
                None,
            ),
        ];

        let () = {
            let mut indexes = self.channels.write().await;
            for channel in public_channels {
                self.channels.create_channel_inner(
                    &mut indexes,
                    channel.into(),
                    false,
                );
            }
        };

        info!(target: LOG_TARGET, "Public channels successfully initialized.",);

        Ok(ExecSuccess::default())
    }

    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatError> {
        fn to_channel_info(ch: &Arc<Channel>) -> ChannelInfo {
            ChannelInfo {
                id: ch.id,
                name: ch.name.to_string(),
                channel_type: ch.channel_type as i32,
                description: ch
                    .description
                    .load()
                    .as_deref()
                    .map(|s| s.to_string()),
                online_users: ch.user_count.val(),
                users: None,
            }
        }

        let channel_indexes = self.channels.read().await;

        let res = GetPublicChannelsResponse {
            channels: channel_indexes
                .public_channels
                .values()
                .map(to_channel_info)
                .collect(),
        };

        Ok(res)
    }
}

#[derive(Clone)]
pub struct ChatServiceRemote {
    pub client: ChatRpcClient<RpcChannel>,
}

impl FromRpcClient for ChatServiceRemote {
    #[inline]
    fn from_client(client: Self::Client) -> Self {
        Self { client }
    }
}

impl RpcClient for ChatServiceRemote {
    type Client = ChatRpcClient<RpcChannel>;

    #[inline]
    fn client(&self) -> Self::Client {
        self.client.clone()
    }
}

impl IntoService<DynChatService> for ChatServiceRemote {
    #[inline]
    fn into_service(self) -> DynChatService {
        Arc::new(self) as DynChatService
    }
}

impl UserSessionsStore for ChatServiceRemote {}

impl NotifyMessagesQueue for ChatServiceRemote {}

impl ChannelStore for ChatServiceRemote {}

#[async_trait]
impl CreateSnapshot<ChatServiceSnapshot> for ChatServiceRemote {
    async fn create_snapshot(&self) -> ChatServiceSnapshot {
        unimplemented!()
    }
}

#[async_trait]
impl ServiceSnapshot for ChatServiceRemote {
    async fn save_service_snapshot(
        &self,
        _: SnapshotType,
        _: &str,
    ) -> Result<(), CreateSnapshotError> {
        unimplemented!()
    }
}

#[async_trait]
impl ChatService for ChatServiceRemote {
    async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<ExecSuccess, ChatError> {
        Ok(self.client().login(request.into_request()).await?.into_inner())
    }

    async fn logout(
        &self,
        query: UserQuery,
        platforms: Platform,
    ) -> Result<ExecSuccess, ChatError> {
        let req = LogoutRequest {
            user_query: Some(query.into()),
            platforms: platforms.bits(),
        }
        .into_request();

        Ok(self.client().logout(req).await?.into_inner())
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatError> {
        Ok(self
            .client()
            .send_message(request.into_request())
            .await?
            .into_inner())
    }

    async fn join_channel(
        &self,
        request: JoinChannelRequest,
    ) -> Result<ExecSuccess, ChatError> {
        Ok(self
            .client()
            .join_channel(request.into_request())
            .await?
            .into_inner())
    }

    async fn leave_channel(
        &self,
        request: LeaveChannelRequest,
    ) -> Result<ExecSuccess, ChatError> {
        Ok(self
            .client()
            .leave_channel(request.into_request())
            .await?
            .into_inner())
    }

    async fn dequeue_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<BanchoPackets, ChatError> {
        Ok(self
            .client()
            .pull_chat_packets(Into::<RawUserQuery>::into(query))
            .await?
            .into_inner())
    }

    async fn load_public_channels(&self) -> Result<ExecSuccess, ChatError> {
        Ok(self
            .client()
            .load_public_channels(LoadPublicChannelsRequest::default())
            .await?
            .into_inner())
    }

    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatError> {
        Ok(self
            .client()
            .get_public_channels(GetPublicChannelsRequest::default())
            .await?
            .into_inner())
    }
}
