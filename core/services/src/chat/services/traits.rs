use crate::chat::*;
use peace_pb::chat::{
    ChannelQuery, ChannelSessionCount, DeleteFromChannelRequest,
    GetPublicChannelsResponse, JoinIntoChannelRequest, LeaveFromChannelRequest,
};
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
    ) -> Result<ChannelSessionCount, ChatServiceError>;

    async fn leave_from_channel(
        &self,
        request: LeaveFromChannelRequest,
    ) -> Result<ChannelSessionCount, ChatServiceError>;

    async fn delete_from_channel(
        &self,
        request: DeleteFromChannelRequest,
    ) -> Result<ChannelSessionCount, ChatServiceError>;
}

#[async_trait]
pub trait ChannelService {
    fn channels(&self) -> &Arc<Channels>;

    async fn initialize_public_channels(&self);

    async fn create(
        &self,
        id: u64,
        name: String,
        channel_type: ChannelType,
        description: Option<String>,
        users: Vec<i32>,
    ) -> Arc<Channel>;

    async fn join_user(
        &self,
        query: &ChannelQuery,
        user_id: i32,
        platforms: Vec<SessionPlatform>,
    ) -> Option<ChannelMetadata>;

    async fn leave_user(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
        platforms: &[SessionPlatform],
    ) -> Option<ChannelMetadata>;

    async fn delete_user(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
    ) -> Option<ChannelMetadata>;

    async fn delete(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn get(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn exists(&self, query: &ChannelQuery) -> bool;

    async fn clear(&self);

    fn len(&self) -> usize;
}
