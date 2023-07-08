use crate::chat::*;
use peace_pb::{
    bancho_state::{BanchoPackets, UserQuery},
    base::ExecSuccess,
    chat::*,
};
use std::sync::Arc;
use tonic::async_trait;

pub type DynChatService = Arc<dyn ChatService + Send + Sync>;
pub type DynChannelService = Arc<dyn ChannelService + Send + Sync>;
pub type DynQueueService = Arc<dyn QueueService + Send + Sync>;

#[async_trait]
pub trait ProcessBanchoMessage {
    async fn process_bancho_message(
        &self,
        sender_id: i32,
        message: String,
        target: ChatMessageTarget,
    ) -> Result<SendMessageResponse, ChatServiceError>;
}

#[async_trait]
pub trait ChatService:
    CreateQueue
    + RemoveQueue
    + GetPublicChannels
    + AddUserIntoChannel
    + RemoveUserPlatformsFromChannel
    + RemoveUserFromChannel
    + SendMessage
    + PullChatPackets
{
}

#[async_trait]
pub trait CreateQueue {
    async fn create_queue(
        &self,
        request: CreateQueueRequest,
    ) -> Result<ExecSuccess, ChatServiceError>;
}

#[async_trait]
pub trait RemoveQueue {
    async fn remove_queue(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, ChatServiceError>;
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
    ) -> Result<ExecSuccess, ChatServiceError>;
}

#[async_trait]
pub trait RemoveUserPlatformsFromChannel {
    async fn remove_user_platforms_from_channel(
        &self,
        request: RemoveUserPlatformsFromChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError>;
}

#[async_trait]
pub trait ChannelUpdateForBancho {
    async fn channel_update_for_bancho(&self, channel: &Arc<Channel>);
}

#[async_trait]
pub trait ChannelHandleForBanchoUser {
    async fn channel_handle_for_bancho_user(
        &self,
        channel: &Arc<Channel>,
        user_id: i32,
    );
}

#[async_trait]
pub trait RemoveUserFromChannel {
    async fn remove_user_from_channel(
        &self,
        request: RemoveUserFromChannelRequest,
    ) -> Result<ExecSuccess, ChatServiceError>;
}

#[async_trait]
pub trait SendMessage {
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatServiceError>;
}

#[async_trait]
pub trait PullChatPackets {
    async fn pull_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<BanchoPackets, ChatServiceError>;
}

#[async_trait]
pub trait ChannelService:
    ChannelStore
    + UserChannelIndex
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

pub trait UserChannelIndex {
    fn user_channels(&self) -> &Arc<UserChannels>;
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
        platforms: Platform,
    ) -> Option<Arc<Channel>>;
}

#[async_trait]
pub trait RemoveUserPlatforms {
    async fn remove_user_platforms(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
        platforms: Platform,
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

#[async_trait]
pub trait QueueService: UserSessionsStore {
    async fn create_queue(
        &self,
        request: CreateQueueRequest,
    ) -> Result<ExecSuccess, ChatServiceError>;

    async fn remove_queue(
        &self,
        query: &UserQuery,
    ) -> Result<ExecSuccess, ChatServiceError>;
}

#[async_trait]
pub trait UserSessionsStore {
    fn user_sessions(&self) -> &Arc<UserSessions>;
}
