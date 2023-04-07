use super::{ChatService, DynChatService};
use crate::{bancho_state::DynBanchoStateService, chat::*};
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
    atomic::I64,
    cache::{CachedAtomic, CachedValue},
};

pub const DEFAULT_CHANNEL_CACHE_EXPIRES: I64 = I64::new(300);

#[derive(Clone)]
pub enum ChatServiceImpl {
    Remote(ChatServiceRemote),
    Local(ChatServiceLocal),
}

impl ChatServiceImpl {
    pub fn into_service(self) -> DynChatService {
        Arc::new(self) as DynChatService
    }

    pub fn remote(client: ChatRpcClient<Channel>) -> Self {
        Self::Remote(ChatServiceRemote {
            client,
            public_channel_info: PublicChannelInfo(CachedAtomic::new(
                DEFAULT_CHANNEL_CACHE_EXPIRES,
            ))
            .into(),
        })
    }

    pub fn local(
        channel_service: DynChannelService,
        bancho_state_service: DynBanchoStateService,
    ) -> Self {
        Self::Local(ChatServiceLocal::new(
            channel_service,
            bancho_state_service,
        ))
    }
}

#[derive(Clone)]
pub struct ChatServiceRemote {
    client: ChatRpcClient<Channel>,
    public_channel_info: Arc<PublicChannelInfo>,
}

#[derive(Clone)]
pub struct ChatServiceLocal {
    #[allow(dead_code)]
    channel_service: DynChannelService,
    #[allow(dead_code)]
    bancho_state_service: DynBanchoStateService,
}

impl ChatServiceRemote {
    pub fn client(&self) -> ChatRpcClient<Channel> {
        self.client.clone()
    }
}

impl ChatServiceLocal {
    pub fn new(
        channel_service: DynChannelService,
        bancho_state_service: DynBanchoStateService,
    ) -> Self {
        Self { channel_service, bancho_state_service }
    }
}

#[async_trait]
impl ChatService for ChatServiceImpl {
    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatServiceError> {
        match self {
            Self::Remote(svc) => Ok(GetPublicChannelsResponse {
                channels: svc.public_channel_info.fetch(svc).await?,
            }),
            Self::Local(svc) => {
                let indexes =
                    svc.channel_service.channels().indexes.read().await;

                Ok(GetPublicChannelsResponse {
                    channels: indexes
                        .channel_public
                        .values()
                        .map(|channel| channel.rpc_channel_info())
                        .collect(),
                })
            },
        }
    }

    async fn join_into_channel(
        &self,
        request: JoinIntoChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .join_into_channel(request)
                .await
                .map_err(ChatServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let JoinIntoChannelRequest {
                    channel_query,
                    user_id,
                    platforms,
                } = request;

                let channel_query = channel_query
                    .ok_or(ChatServiceError::InvalidArgument)?
                    .into();
                let platforms =
                    platforms.map(|p| PlatformsLoader::load_from_vec(p.value));

                let channel = svc
                    .channel_service
                    .join_user(&channel_query, user_id, platforms)
                    .await
                    .ok_or(ChatServiceError::ChannelNotExists)?;

                Ok(channel.rpc_channel_info())
            },
        }
    }

    async fn leave_from_channel(
        &self,
        request: LeaveFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .leave_from_channel(request)
                .await
                .map_err(ChatServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let LeaveFromChannelRequest {
                    channel_query,
                    user_id,
                    platforms,
                } = request;

                let channel_query = channel_query
                    .ok_or(ChatServiceError::InvalidArgument)?
                    .into();
                let platforms =
                    platforms.map(|p| PlatformsLoader::load_from_vec(p.value));

                let channel = svc
                    .channel_service
                    .leave_user(&channel_query, &user_id, platforms.as_deref())
                    .await
                    .ok_or(ChatServiceError::ChannelNotExists)?;

                Ok(channel.rpc_channel_info())
            },
        }
    }

    async fn delete_from_channel(
        &self,
        request: DeleteFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .delete_from_channel(request)
                .await
                .map_err(ChatServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let DeleteFromChannelRequest { channel_query, user_id } =
                    request;

                let channel_query = channel_query
                    .ok_or(ChatServiceError::InvalidArgument)?
                    .into();

                let channel = svc
                    .channel_service
                    .delete_user(&channel_query, &user_id)
                    .await
                    .ok_or(ChatServiceError::ChannelNotExists)?;

                Ok(channel.rpc_channel_info())
            },
        }
    }

    async fn send_message_to(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError> {
        match self {
            Self::Remote(svc) => svc
                .client()
                .send_message_to(request)
                .await
                .map_err(ChatServiceError::RpcError)
                .map(|resp| resp.into_inner()),
            Self::Local(svc) => {
                let SendMessageRequest { sender_id, message, target, platform } =
                    request;

                let platform = Platform::from_u16(platform as u16)
                    .ok_or(ChatServiceError::InvalidArgument)?;
                let target = Into::<ChatMessageTarget>::into(
                    target.ok_or(ChatServiceError::InvalidArgument)?,
                );

                match platform {
                    Platform::Bancho => match target {
                        ChatMessageTarget::ChannelId(_) => todo!(),
                        ChatMessageTarget::ChannelName(target_channel_name) => {
                            let channel = svc
                                .channel_service
                                .get(&ChannelQuery::ChannelName(
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

                            svc.bancho_state_service
                                .batch_enqueue_bancho_packets(
                                    BatchEnqueueBanchoPacketsRequest {
                                        targets,
                                        packets:
                                            bancho_packets::server::SendMessage::pack(
                                                "test1".into(),
                                                message.as_str().into(),
                                                target_channel_name
                                                    .as_str()
                                                    .into(),
                                                sender_id,
                                            ),
                                    },
                                )
                                .await
                                .unwrap();
                        },
                        ChatMessageTarget::UserId(_) => todo!(),
                        ChatMessageTarget::Username(target_username) => {
                            let sender = svc
                                .bancho_state_service
                                .get_user_session_with_fields(
                                    RawUserQueryWithFields {
                                        user_query: Some(
                                            UserQuery::UserId(sender_id).into(),
                                        ),
                                        fields: UserSessionFields::Username
                                            .bits(),
                                    },
                                )
                                .await
                                .unwrap();

                            svc.bancho_state_service
                                .enqueue_bancho_packets(
                                    EnqueueBanchoPacketsRequest {
                                        target: Some(
                                            BanchoPacketTarget::Username(
                                                target_username.to_owned(),
                                            )
                                            .into(),
                                        ),
                                        packets:
                                            bancho_packets::server::SendMessage::pack(
                                                sender.username.unwrap().into(),
                                                message.into(),
                                                target_username.into(),
                                                sender_id,
                                            ),
                                    },
                                )
                                .await
                                .unwrap();
                        },
                        ChatMessageTarget::UsernameUnicode(_) => todo!(),
                    },
                    Platform::Lazer => todo!(),
                    Platform::Web => todo!(),
                };

                Ok(SendMessageResponse { message_id: 0 })
            },
        }
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
                context.public_channel_info.set(Some(channels.clone().into()));
                channels
            })
    }

    #[inline]
    async fn fetch(&self, context: &Self::Context) -> Self::Output {
        Ok(match context.public_channel_info.get() {
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
