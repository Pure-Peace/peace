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

    async fn add_user_into_channel(
        &self,
        request: AddUserIntoChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;

    async fn remove_user_platforms_from_channel(
        &self,
        request: RemoveUserPlatformsFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;

    async fn remove_user_from_channel(
        &self,
        request: RemoveUserFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError>;
}

#[async_trait]
pub trait ChannelService {
    fn channels(&self) -> &Arc<Channels>;

    async fn initialize_public_channels(&self);

    async fn add_channel(
        &self,
        metadata: ChannelMetadata,
        users: Vec<i32>,
    ) -> Arc<Channel>;

    async fn add_user(
        &self,
        query: &ChannelQuery,
        user_id: i32,
        platforms: Option<Vec<Platform>>,
    ) -> Option<Arc<Channel>>;

    async fn remove_user_platforms(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
        platforms: Option<&[Platform]>,
    ) -> Option<Arc<Channel>>;

    async fn remove_user(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
    ) -> Option<Arc<Channel>>;

    async fn remove_channel(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn get_channel(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn is_channel_exists(&self, query: &ChannelQuery) -> bool;

    async fn clear_all_channels(&self);

    fn channel_count(&self) -> usize;
}
