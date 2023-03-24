use crate::chat::*;
use peace_pb::chat::*;
use std::sync::Arc;
use tonic::async_trait;

pub type DynChatService = Arc<dyn ChatService + Send + Sync>;
pub type DynChannelService = Arc<dyn ChannelService + Send + Sync>;

#[async_trait]
pub trait ChatService {
    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatServiceError>;

    async fn join_into_channel(
        &self,
        request: JoinIntoChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;

    async fn leave_from_channel(
        &self,
        request: LeaveFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;

    async fn delete_from_channel(
        &self,
        request: DeleteFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;
}

#[async_trait]
pub trait ChannelService {
    fn channels(&self) -> &Arc<Channels>;

    async fn initialize_public_channels(&self);

    async fn create(
        &self,
        metadata: ChannelMetadata,
        users: Vec<i32>,
    ) -> Arc<Channel>;

    async fn join_user(
        &self,
        query: &ChannelQuery,
        user_id: i32,
        platforms: Option<Vec<Platform>>,
    ) -> Option<Arc<Channel>>;

    async fn leave_user(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
        platforms: Option<&[Platform]>,
    ) -> Option<Arc<Channel>>;

    async fn delete_user(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
    ) -> Option<Arc<Channel>>;

    async fn delete(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn get(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn exists(&self, query: &ChannelQuery) -> bool;

    async fn clear(&self);

    fn len(&self) -> usize;
}
