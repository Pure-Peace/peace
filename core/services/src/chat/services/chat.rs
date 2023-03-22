use super::{ChatService, DynChatService};
use crate::{
    bancho_state::DynBanchoStateService,
    chat::{ChatServiceError, DynChannelService},
};
use async_trait::async_trait;
use derive_deref::Deref;
use peace_pb::chat::{
    chat_rpc_client::ChatRpcClient, ChannelInfo, GetPublicChannelsRequest,
    GetPublicChannelsResponse,
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
                            length: channel.users.read().await.len() as u32,
                            users: None,
                        }
                    }),
                )
                .await;

                Ok(GetPublicChannelsResponse { channels })
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
