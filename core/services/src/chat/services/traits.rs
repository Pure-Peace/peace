use crate::chat::*;
use peace_pb::chat::*;
use std::sync::Arc;
use tonic::async_trait;

pub type DynChatService = Arc<dyn ChatService + Send + Sync>;
pub type DynChannelService = Arc<dyn ChannelService + Send + Sync>;

#[async_trait]
pub trait ChatService:
    GetPublicChannels
    + AddUserIntoChannel
    + RemoveUserPlatformsFromChannel
    + RemoveUserFromChannel
    + SendMessage
{
}

#[async_trait]
pub trait GetPublicChannels {
    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatServiceError>;
}
#[async_trait]
pub trait AddUserIntoChannel {
    async fn add_user_into_channel(
        &self,
        request: AddUserIntoChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;
}

#[async_trait]
pub trait RemoveUserPlatformsFromChannel {
    async fn remove_user_platforms_from_channel(
        &self,
        request: RemoveUserPlatformsFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;
}

#[async_trait]
pub trait RemoveUserFromChannel {
    async fn remove_user_from_channel(
        &self,
        request: RemoveUserFromChannelRequest,
    ) -> Result<ChannelInfo, ChatServiceError>;
}

#[async_trait]
pub trait SendMessage {
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError>;
}

#[async_trait]
pub trait ChannelService:
    ChannelStore
    + InitializePublicChannels
    + AddChannel
    + AddUser
    + RemoveUserPlatforms
    + RemoveUser
    + RemoveChannel
    + GetChannel
    + IsChannelExists
    + ClearAllChannels
    + ChannelCount
{
}

pub trait ChannelStore {
    fn channels(&self) -> &Arc<Channels>;
}

#[async_trait]
pub trait InitializePublicChannels {
    async fn initialize_public_channels(&self);
}

#[async_trait]
pub trait AddChannel {
    async fn add_channel(
        &self,
        metadata: ChannelMetadata,
        users: Vec<i32>,
    ) -> Arc<Channel>;
}

#[async_trait]
pub trait AddUser {
    async fn add_user(
        &self,
        query: &ChannelQuery,
        user_id: i32,
        platforms: Option<Vec<Platform>>,
    ) -> Option<Arc<Channel>>;
}

#[async_trait]
pub trait RemoveUserPlatforms {
    async fn remove_user_platforms(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
        platforms: Option<&[Platform]>,
    ) -> Option<Arc<Channel>>;
}

#[async_trait]
pub trait RemoveUser {
    async fn remove_user(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
    ) -> Option<Arc<Channel>>;
}

#[async_trait]
pub trait RemoveChannel {
    async fn remove_channel(
        &self,
        query: &ChannelQuery,
    ) -> Option<Arc<Channel>>;
}

#[async_trait]
pub trait GetChannel {
    async fn get_channel(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;
}

#[async_trait]
pub trait IsChannelExists {
    async fn is_channel_exists(&self, query: &ChannelQuery) -> bool;
}

#[async_trait]
pub trait ClearAllChannels {
    async fn clear_all_channels(&self);
}

pub trait ChannelCount {
    fn channel_count(&self) -> usize;
}
