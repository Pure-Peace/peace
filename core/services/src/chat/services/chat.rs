use super::{ChatService, DynChatService};
use crate::{
    bancho_state::DynBanchoStateService,
    chat::{
        ChannelMetadata, ChatServiceError, DynChannelService, SessionPlatform,
        SessionPlatforms,
    },
};
use async_trait::async_trait;
use derive_deref::Deref;
use peace_pb::{
    bancho_state::{
        BanchoPacketTarget, BroadcastBanchoPacketsRequest,
        EnqueueBanchoPacketsRequest,
    },
    chat::{
        chat_rpc_client::ChatRpcClient, ChannelInfo, ChannelSessionCount,
        DeleteFromChannelRequest, GetPublicChannelsRequest,
        GetPublicChannelsResponse, JoinIntoChannelRequest,
        LeaveFromChannelRequest,
    },
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

                let channels = futures::future::join_all(
                    indexes.channel_public.values().map(|channel| async {
                        ChannelInfo {
                            id: channel.id,
                            name: channel.name.to_string(),
                            channel_type: channel.channel_type as i32,
                            description: channel
                                .description
                                .load()
                                .as_ref()
                                .map(|s| s.to_string()),
                            length: channel.read().await.len() as u32,
                            users: None,
                        }
                    }),
                )
                .await;

                Ok(GetPublicChannelsResponse { channels })
            },
        }
    }

    async fn join_into_channel(
        &self,
        request: JoinIntoChannelRequest,
    ) -> Result<ChannelSessionCount, ChatServiceError> {
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
                let platforms = SessionPlatforms::from(platforms);

                let contains_bancho =
                    platforms.contains(&SessionPlatform::Bancho);

                let ChannelMetadata { name, session_count, .. } = svc
                    .channel_service
                    .join_user(&channel_query, user_id, platforms.into_inner())
                    .await
                    .ok_or(ChatServiceError::ChannelNotExists)?;

                if contains_bancho {
                    svc.bancho_state_service
                        .enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
                            target: Some(
                                BanchoPacketTarget::UserId(user_id).into(),
                            ),
                            packets: bancho_packets::server::ChannelJoin::pack(
                                name.as_str().into(),
                            ),
                        })
                        .await?;

                    svc.bancho_state_service
                        .broadcast_bancho_packets(
                            BroadcastBanchoPacketsRequest {
                                packets:
                                    bancho_packets::server::ChannelInfo::pack(
                                        name.into(),
                                        "todo".into(),
                                        session_count as i16,
                                    ),
                            },
                        )
                        .await?;
                }

                Ok(ChannelSessionCount { session_count: session_count as u32 })
            },
        }
    }

    async fn leave_from_channel(
        &self,
        request: LeaveFromChannelRequest,
    ) -> Result<ChannelSessionCount, ChatServiceError> {
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
                let platforms = SessionPlatforms::from(platforms);

                let contains_bancho =
                    platforms.contains(&SessionPlatform::Bancho);

                let ChannelMetadata { name, session_count, .. } = svc
                    .channel_service
                    .leave_user(
                        &channel_query,
                        &user_id,
                        &platforms.into_inner(),
                    )
                    .await
                    .ok_or(ChatServiceError::ChannelNotExists)?;

                if contains_bancho {
                    svc.bancho_state_service
                        .enqueue_bancho_packets(EnqueueBanchoPacketsRequest {
                            target: Some(
                                BanchoPacketTarget::UserId(user_id).into(),
                            ),
                            packets: bancho_packets::server::ChannelKick::pack(
                                name.as_str().into(),
                            ),
                        })
                        .await?;

                    svc.bancho_state_service
                        .broadcast_bancho_packets(
                            BroadcastBanchoPacketsRequest {
                                packets:
                                    bancho_packets::server::ChannelInfo::pack(
                                        name.into(),
                                        "todo".into(),
                                        session_count as i16,
                                    ),
                            },
                        )
                        .await?;
                }

                Ok(ChannelSessionCount { session_count: session_count as u32 })
            },
        }
    }

    async fn delete_from_channel(
        &self,
        request: DeleteFromChannelRequest,
    ) -> Result<ChannelSessionCount, ChatServiceError> {
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

                let ChannelMetadata { session_count, .. } = svc
                    .channel_service
                    .delete_user(&channel_query, &user_id)
                    .await
                    .ok_or(ChatServiceError::ChannelNotExists)?;

                Ok(ChannelSessionCount { session_count: session_count as u32 })
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
