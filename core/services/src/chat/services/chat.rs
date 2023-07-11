use crate::{
    bancho_state::{BanchoMessageQueue, BanchoPacketsQueue, Packet},
    chat::*,
    users::Session,
    FromRpcClient, IntoService, RpcClient,
};
use async_trait::async_trait;
use bancho_packets::server;
use chat::traits::{ChatService, DynChatService};
use chrono::Utc;
use peace_db::{peace::Peace, DbConnection};
use peace_domain::bancho_state::CreateSessionDto;
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
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;
use tonic::{transport::Channel as RpcChannel, IntoRequest};
use tools::{
    atomic::{AtomicOperation, AtomicValue},
    message_queue::ReceivedMessages,
    Ulid,
};

#[derive(Default, Clone)]
pub struct ChatServiceImpl {
    pub conn: DbConnection<Peace>,
    pub user_sessions: Arc<UserSessions>,
    pub notify_queue: Arc<Mutex<BanchoMessageQueue>>,
    pub channels: Arc<Channels>,
}

impl ChatServiceImpl {
    #[inline]
    pub fn new(conn: DbConnection<Peace>) -> Self {
        Self { conn, ..Default::default() }
    }

    #[inline]
    pub fn into_service(self) -> DynChatService {
        Arc::new(self) as DynChatService
    }
}

#[async_trait]
impl ChatService for ChatServiceImpl {
    async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        let LoginRequest {
            user_id,
            username,
            username_unicode,
            privileges,
            platforms,
        } = request;

        let platforms = Platform::from(platforms);

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

            Some(BanchoPacketsQueue::new(channel_packets).into())
        } else {
            None
        };

        let extends = ChatSessionExtend::new(platforms, bancho_chat_ext, None);

        let session = Session::new(CreateSessionDto {
            user_id,
            username,
            username_unicode,
            privileges,
            extends,
        });

        let _session = self.user_sessions.create(session).await;

        Ok(ExecSuccess::default())
    }

    async fn logout(
        &self,
        query: UserQuery,
        remove_platforms: Platform,
    ) -> Result<ExecSuccess, ChatServiceError> {
        if remove_platforms.is_all() || remove_platforms.is_none() {
            self.user_sessions.delete(&query).await;
            return Ok(ExecSuccess::default());
        }

        let session = match self.user_sessions.get(&query).await {
            Some(session) => session,
            None => return Ok(ExecSuccess::default()),
        };

        let curr_platforms = session.extends.platforms.val();

        // Logout from bancho
        if curr_platforms.contains(Platform::Bancho)
            && remove_platforms.contains(Platform::Bancho)
        {
            todo!("Logout from bancho")
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
            self.user_sessions.delete(&query).await;
            return Ok(ExecSuccess::default());
        }

        session.extends.platforms.set(platforms.into());

        Ok(ExecSuccess::default())
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError> {
        let SendMessageRequest { sender, message, target } = request;

        let sender_query = sender
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_user_query()?;

        let sender = match self.user_sessions.get(&sender_query).await {
            Some(sender) => sender,
            None => {
                todo!("sender not login")
            },
        };

        let target = target
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_message_target()?;

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

                // push msg into channel packets queue
                channel.message_queue.lock().await.push_message_excludes(
                    Packet::Ptr(
                        server::SendMessage::pack(
                            sender.username.load().as_ref().into(),
                            message.into(),
                            channel.name.load().as_ref().into(),
                            sender.user_id,
                        )
                        .into(),
                    ),
                    [sender.user_id],
                    None,
                );
            },
            ChatMessageTarget::User(user_query) => {
                // get target user session
                let target_user =
                    match self.user_sessions.get(&user_query).await {
                        Some(target_user) => target_user,
                        None => {
                            todo!("target not login")
                        },
                    };

                // push msg packet if target user's bancho packets queue is exists
                if let Some(bancho_ext) =
                    target_user.extends.bancho_ext.as_ref()
                {
                    bancho_ext
                        .packets_queue
                        .push_packet(
                            server::SendMessage::pack(
                                sender.username.load().as_ref().into(),
                                message.into(),
                                target_user.username.load().as_ref().into(),
                                sender.user_id,
                            )
                            .into(),
                        )
                        .await;
                }
            },
        }

        todo!()
    }

    async fn join_channel(
        &self,
        request: JoinChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        let JoinChannelRequest { channel_query, user_query } = request;

        let user_query = user_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_user_query()?;

        let channel_query = channel_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_channel_query()?;

        let session = match self.user_sessions.get(&user_query).await {
            Some(session) => session,
            None => {
                todo!("not login")
            },
        };

        let channel = match self.channels.get_channel(&channel_query).await {
            Some(channel) => channel,
            None => {
                todo!("channel not exists")
            },
        };

        // add user into channel
        channel.users.write().await.entry(session.user_id).or_insert_with(
            || {
                channel.user_count.add(1);
                Some(Arc::downgrade(&session))
            },
        );

        session
            .extends
            .joined_channels
            .write()
            .await
            .entry(channel.id)
            .or_insert_with(|| {
                session.extends.channel_count.add(1);
                JoinedChannel {
                    ptr: Arc::downgrade(&channel),
                    message_index: Ulid::default().into(),
                    joined_time: Utc::now(),
                }
                .into()
            });

        // notify to user's bancho client if possible
        if let Some(bancho_ext) = session.extends.bancho_ext.as_ref() {
            bancho_ext
                .packets_queue
                .push_packet(
                    server::ChannelJoin::pack(
                        channel.name.load().as_ref().into(),
                    )
                    .into(),
                )
                .await;
        }

        // update channel info
        self.notify_queue.lock().await.push_message(
            Packet::Ptr(
                server::ChannelInfo::pack(
                    channel.name.load().as_ref().into(),
                    channel
                        .description
                        .load()
                        .as_deref()
                        .map(|s| s.to_owned())
                        .unwrap_or_default()
                        .into(),
                    channel.user_count.val() as i16,
                )
                .into(),
            ),
            None,
        );

        Ok(ExecSuccess::default())
    }

    async fn leave_channel(
        &self,
        request: LeaveChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        let LeaveChannelRequest { channel_query, user_query } = request;

        let user_query = user_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_user_query()?;

        let session = match self.user_sessions.get(&user_query).await {
            Some(session) => session,
            None => {
                todo!("not login")
            },
        };

        let channel_query = channel_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_channel_query()?;

        let channel = match self.channels.get_channel(&channel_query).await {
            Some(channel) => channel,
            None => {
                todo!("channel not exists")
            },
        };

        // remove user from channel
        if channel.users.write().await.remove(&session.user_id).is_some() {
            channel.user_count.sub(1);
        }

        if session
            .extends
            .joined_channels
            .write()
            .await
            .remove(&channel.id)
            .is_some()
        {
            session.extends.channel_count.sub(1);
        }

        // notify to user's bancho client if possible
        if let Some(bancho_ext) = session.extends.bancho_ext.as_ref() {
            bancho_ext
                .packets_queue
                .push_packet(
                    server::ChannelKick::pack(
                        channel.name.load().as_ref().into(),
                    )
                    .into(),
                )
                .await;
        }

        // update channel info
        self.notify_queue.lock().await.push_message(
            Packet::Ptr(
                server::ChannelInfo::pack(
                    channel.name.load().as_ref().into(),
                    channel
                        .description
                        .load()
                        .as_deref()
                        .map(|s| s.to_owned())
                        .unwrap_or_default()
                        .into(),
                    channel.user_count.val() as i16,
                )
                .into(),
            ),
            None,
        );

        Ok(ExecSuccess::default())
    }

    async fn dequeue_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<BanchoPackets, ChatServiceError> {
        let session = match self.user_sessions.get(&query).await {
            Some(session) => session,
            None => {
                todo!("not login")
            },
        };

        let bancho_ext = match session.extends.bancho_ext.as_ref() {
            Some(bancho_ext) => bancho_ext,
            None => todo!("invalid call"),
        };

        let mut data = Vec::new();

        // receive global notify from queue
        if let Some(ReceivedMessages { messages, last_msg_id }) =
            self.notify_queue.lock().await.receive_messages(
                &session.user_id,
                &bancho_ext.notify_index.load(),
                None,
            )
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
            match joined_channel.ptr.upgrade() {
                Some(channel) => {
                    if let Some(ReceivedMessages { messages, last_msg_id }) =
                        channel.message_queue.lock().await.receive_messages(
                            &session.user_id,
                            &joined_channel.message_index.load(),
                            None,
                        )
                    {
                        for packet in messages {
                            data.extend(packet);
                        }

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

    async fn load_public_channels(
        &self,
    ) -> Result<ExecSuccess, ChatServiceError> {
        const LOG_TARGET: &str = "chat::channel::initialize_public_channels";

        // todo: load public channels from database
        let public_channels = vec![
            Channel::new(
                0,
                "#osu".to_string().into(),
                ChannelType::Public,
                Some("default channel".to_string()).into(),
                None,
            ),
            Channel::new(
                1,
                "#peace".to_string().into(),
                ChannelType::Public,
                Some("peace channel".to_string()).into(),
                None,
            ),
        ];

        let () = {
            let mut indexes = self.channels.write().await;
            for channel in public_channels {
                self.channels
                    .create_channel_inner(&mut indexes, channel.into());
            }
        };

        info!(target: LOG_TARGET, "Public channels successfully initialized.",);

        Ok(ExecSuccess::default())
    }

    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatServiceError> {
        let channel_indexes = self.channels.read().await;

        Ok(GetPublicChannelsResponse {
            channels: channel_indexes
                .public_channels
                .values()
                .map(|channel| ChannelInfo {
                    id: channel.id,
                    name: channel.name.to_string(),
                    channel_type: channel.channel_type as i32,
                    description: channel
                        .description
                        .load()
                        .as_ref()
                        .map(|s| s.to_string()),
                    online_users: channel.user_count.val(),
                    users: None,
                })
                .collect(),
        })
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

#[async_trait]
impl ChatService for ChatServiceRemote {
    async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        self.client()
            .login(request.into_request())
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }

    async fn logout(
        &self,
        query: UserQuery,
        platforms: Platform,
    ) -> Result<ExecSuccess, ChatServiceError> {
        self.client()
            .logout(
                LogoutRequest {
                    user_query: Some(query.into()),
                    platforms: platforms.bits(),
                }
                .into_request(),
            )
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError> {
        self.client()
            .send_message(request.into_request())
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }

    async fn join_channel(
        &self,
        request: JoinChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        self.client()
            .join_channel(request.into_request())
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }

    async fn leave_channel(
        &self,
        request: LeaveChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        self.client()
            .leave_channel(request.into_request())
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }

    async fn dequeue_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<BanchoPackets, ChatServiceError> {
        self.client()
            .pull_chat_packets(Into::<RawUserQuery>::into(query))
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }

    async fn load_public_channels(
        &self,
    ) -> Result<ExecSuccess, ChatServiceError> {
        self.client()
            .load_public_channels(LoadPublicChannelsRequest::default())
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }

    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatServiceError> {
        self.client()
            .get_public_channels(GetPublicChannelsRequest::default())
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
