use crate::{
    bancho_state::{BanchoMessageQueue, BanchoPacketsQueue},
    chat::*,
    users::Session,
    FromRpcClient, IntoService, RpcClient,
};
use async_trait::async_trait;
use bancho_packets::server;
use chat::traits::{ChatService, DynChatService};
use peace_db::{peace::Peace, DbConnection};
use peace_domain::bancho_state::CreateSessionDto;
use peace_pb::{
    bancho_state::{BanchoPackets, RawUserQuery, UserQuery},
    base::ExecSuccess,
    chat::{
        chat_rpc_client::ChatRpcClient, ChannelInfo, GetPublicChannelsRequest,
        GetPublicChannelsResponse, JoinChannelRequest, LeaveChannelRequest,
        LoginRequest, LogoutRequest, SendMessageRequest, SendMessageResponse,
    },
};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;
use tonic::{transport::Channel as RpcChannel, IntoRequest};
use tools::atomic::AtomicValue;

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

        let extends = ChatExtend::new(platforms, bancho_chat_ext);

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
            todo!()
        }

        // TODO: part from other platforms
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
        todo!()
    }

    async fn join_channel(
        &self,
        request: JoinChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        let JoinChannelRequest { channel_query, user_id } = request;

        let channel_query = channel_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_channel_query()?;

        let channel = match self.channels.get_channel(&channel_query).await {
            Some(channel) => channel,
            None => return Ok(ExecSuccess::default()),
        };

        channel.users.write().await.insert(user_id);

        todo!()
    }

    async fn leave_channel(
        &self,
        request: LeaveChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError> {
        todo!()
    }

    async fn dequeue_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<BanchoPackets, ChatServiceError> {
        todo!()
    }

    async fn load_public_channels(&self) -> Result<(), ChatServiceError> {
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

        Ok(())
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

    async fn load_public_channels(&self) -> Result<(), ChatServiceError> {
        todo!()
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
