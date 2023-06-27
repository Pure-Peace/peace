use super::{ChatService, DynChatService};
use crate::{
    bancho_state::DynBanchoStateService, chat::*, message::DynMessageService,
    FromRpcClient, IntoService, RpcClient,
};
use async_trait::async_trait;
use derive_deref::Deref;
use num_traits::FromPrimitive;
use peace_pb::{
    bancho_state::{
        BanchoPacketTarget, BatchEnqueueBanchoPacketsRequest,
        EnqueueBanchoPacketsRequest, RawBanchoPacketTarget,
        RawUserQueryWithFields, UserQuery, UserSessionFields,
    },
    chat::{chat_rpc_client::ChatRpcClient, *},
};
use std::sync::Arc;
use tonic::transport::Channel;
use tools::{
    atomic::U64,
    cache::{CachedAtomic, CachedValue},
};

pub const DEFAULT_CHANNEL_CACHE_EXPIRES: U64 = U64::new(300);

#[derive(Clone)]
pub struct ChatServiceImpl {
    pub channel_service: DynChannelService,
    pub bancho_state_service: DynBanchoStateService,
    pub message_service: DynMessageService,
}

impl ChatServiceImpl {
    pub fn new(
        channel_service: DynChannelService,
        bancho_state_service: DynBanchoStateService,
        message_service: DynMessageService,
    ) -> Self {
        Self { channel_service, bancho_state_service, message_service }
    }

    pub fn into_service(self) -> DynChatService {
        Arc::new(self) as DynChatService
    }
}

#[async_trait]
impl ChatService for ChatServiceImpl {}

#[async_trait]
impl GetPublicChannels for ChatServiceImpl {
    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatServiceError> {
        let indexes = self.channel_service.channels().indexes.read().await;

        Ok(GetPublicChannelsResponse {
            channels: indexes
                .channel_public
                .values()
                .map(|channel| channel.rpc_channel_info())
                .collect(),
        })
    }
}

#[async_trait]
impl AddUserIntoChannel for ChatServiceImpl {
    async fn add_user_into_channel(
        &self,
        request: AddUserIntoChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        let AddUserIntoChannelRequest { channel_query, user_id, platforms } =
            request;

        let channel_query = channel_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_channel_query()?;
        let platforms =
            platforms.map(|p| PlatformsLoader::load_from_vec(p.value));

        let channel = self
            .channel_service
            .add_user(&channel_query, user_id, platforms)
            .await
            .ok_or(ChatServiceError::ChannelNotExists)?;

        Ok(channel.rpc_channel_info())
    }
}
#[async_trait]
impl RemoveUserPlatformsFromChannel for ChatServiceImpl {
    async fn remove_user_platforms_from_channel(
        &self,
        request: RemoveUserPlatformsFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        let RemoveUserPlatformsFromChannelRequest {
            channel_query,
            user_id,
            platforms,
        } = request;

        let channel_query = channel_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_channel_query()?;
        let platforms =
            platforms.map(|p| PlatformsLoader::load_from_vec(p.value));

        let channel = self
            .channel_service
            .remove_user_platforms(
                &channel_query,
                &user_id,
                platforms.as_deref(),
            )
            .await
            .ok_or(ChatServiceError::ChannelNotExists)?;

        Ok(channel.rpc_channel_info())
    }
}
#[async_trait]
impl RemoveUserFromChannel for ChatServiceImpl {
    async fn remove_user_from_channel(
        &self,
        request: RemoveUserFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        let RemoveUserFromChannelRequest { channel_query, user_id } = request;

        let channel_query = channel_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_channel_query()?;

        let channel = self
            .channel_service
            .remove_user(&channel_query, &user_id)
            .await
            .ok_or(ChatServiceError::ChannelNotExists)?;

        Ok(channel.rpc_channel_info())
    }
}
#[async_trait]
impl SendMessage for ChatServiceImpl {
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError> {
        let SendMessageRequest { sender_id, message, target, platform } =
            request;

        let platform = Platform::from_u16(platform as u16)
            .ok_or(ChatServiceError::InvalidArgument)?;
        let target = target
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_message_target()?;

        self.message_service
            .publish_stream(
                Subjects::Message.to_string(),
                ChatMessage { sender_id, message, platform, target }
                    .to_bytes()
                    .map_err(ChatServiceError::SerializeError)?,
            )
            .await?;

        // TODO: Redo
        /* match platform {
            Platform::Bancho => match target {
                ChatMessageTarget::ChannelId(_) => todo!(),
                ChatMessageTarget::ChannelName(target_channel_name) => {
                    let channel = self
                        .channel_service
                        .get_channel(&ChannelQuery::ChannelName(
                            target_channel_name.to_owned(),
                        ))
                        .await
                        .unwrap();

                    let targets = channel
                        .sessions
                        .indexes
                        .read()
                        .await
                        .bancho
                        .keys()
                        .filter(|user_id| *user_id != &sender_id)
                        .map(|user_id| {
                            BanchoPacketTarget::UserId(*user_id).into()
                        })
                        .collect::<Vec<RawBanchoPacketTarget>>();

                    self.bancho_state_service
                        .batch_enqueue_bancho_packets(
                            BatchEnqueueBanchoPacketsRequest {
                                targets,
                                packets:
                                    bancho_packets::server::SendMessage::pack(
                                        "test1".into(),
                                        message.as_str().into(),
                                        target_channel_name.as_str().into(),
                                        sender_id,
                                    ),
                            },
                        )
                        .await
                        .unwrap();
                },
                ChatMessageTarget::UserId(_) => todo!(),
                ChatMessageTarget::Username(target_username) => {
                    let sender = self
                        .bancho_state_service
                        .get_user_session_with_fields(RawUserQueryWithFields {
                            user_query: Some(
                                UserQuery::UserId(sender_id).into(),
                            ),
                            fields: UserSessionFields::Username.bits(),
                        })
                        .await
                        .unwrap();

                    self.bancho_state_service
                        .enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
                            target: Some(
                                BanchoPacketTarget::Username(
                                    target_username.to_owned(),
                                )
                                .into(),
                            ),
                            packets: bancho_packets::server::SendMessage::pack(
                                sender.username.unwrap().into(),
                                message.into(),
                                target_username.into(),
                                sender_id,
                            ),
                        })
                        .await
                        .unwrap();
                },
                ChatMessageTarget::UsernameUnicode(_) => todo!(),
            },
            Platform::Lazer => todo!(),
            Platform::Web => todo!(),
        }; */

        Ok(SendMessageResponse { message_id: 0 })
    }
}

#[derive(Clone)]
pub struct ChatServiceRemote {
    pub client: ChatRpcClient<Channel>,
    pub info: Arc<PublicChannelInfo>,
}

impl FromRpcClient for ChatServiceRemote {
    #[inline]
    fn from_client(client: Self::Client) -> Self {
        Self {
            client,
            info: PublicChannelInfo(CachedAtomic::new(
                DEFAULT_CHANNEL_CACHE_EXPIRES,
            ))
            .into(),
        }
    }
}

impl RpcClient for ChatServiceRemote {
    type Client = ChatRpcClient<Channel>;

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
impl ChatService for ChatServiceRemote {}

#[async_trait]
impl GetPublicChannels for ChatServiceRemote {
    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatServiceError> {
        Ok(GetPublicChannelsResponse { channels: self.info.fetch(self).await? })
    }
}

#[async_trait]
impl AddUserIntoChannel for ChatServiceRemote {
    async fn add_user_into_channel(
        &self,
        request: AddUserIntoChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        self.client()
            .add_user_into_channel(request)
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl RemoveUserPlatformsFromChannel for ChatServiceRemote {
    async fn remove_user_platforms_from_channel(
        &self,
        request: RemoveUserPlatformsFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        self.client()
            .remove_user_platforms_from_channel(request)
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl RemoveUserFromChannel for ChatServiceRemote {
    async fn remove_user_from_channel(
        &self,
        request: RemoveUserFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        self.client()
            .remove_user_from_channel(request)
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl SendMessage for ChatServiceRemote {
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError> {
        self.client()
            .send_message(request)
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[derive(Deref)]
pub struct PublicChannelInfo(pub CachedAtomic<Vec<ChannelInfo>>);

#[async_trait]
impl CachedValue for PublicChannelInfo {
    type Context = ChatServiceRemote;
    type Output = Result<Vec<ChannelInfo>, ChatServiceError>;

    #[inline]
    async fn fetch_new(&self, context: &Self::Context) -> Self::Output {
        context
            .client()
            .get_public_channels(GetPublicChannelsRequest {})
            .await
            .map_err(ChatServiceError::RpcError)
            .map(|resp| {
                let GetPublicChannelsResponse { channels } = resp.into_inner();
                context.info.set(Some(channels.clone().into()));
                channels
            })
    }

    #[inline]
    async fn fetch(&self, context: &Self::Context) -> Self::Output {
        Ok(match context.info.get() {
            Some(cached_value) => {
                if !cached_value.expired {
                    cached_value.cache.to_vec()
                } else {
                    self.fetch_new(context)
                        .await
                        .unwrap_or(cached_value.cache.to_vec())
                }
            },
            None => self.fetch_new(context).await?,
        })
    }
}
