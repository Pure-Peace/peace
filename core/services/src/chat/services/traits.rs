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
pub type DynChatBackgroundService =
    Arc<dyn ChatBackgroundService + Send + Sync>;

#[async_trait]
pub trait ChatService {
    async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<ExecSuccess, ChatError>;

    async fn logout(
        &self,
        query: UserQuery,
        remove_platforms: Platform,
    ) -> Result<ExecSuccess, ChatError>;

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, ChatError>;

    async fn join_channel(
        &self,
        request: JoinChannelRequest,
    ) -> Result<ExecSuccess, ChatError>;

    async fn leave_channel(
        &self,
        request: LeaveChannelRequest,
    ) -> Result<ExecSuccess, ChatError>;

    async fn dequeue_chat_packets(
        &self,
        query: UserQuery,
    ) -> Result<BanchoPackets, ChatError>;

    async fn load_public_channels(&self) -> Result<ExecSuccess, ChatError>;

    async fn get_public_channels(
        &self,
    ) -> Result<GetPublicChannelsResponse, ChatError>;
}

#[async_trait]
pub trait ChannelService {}

#[async_trait]
pub trait ChatBackgroundService {
    fn start_all(&self, configs: ChatBackgroundServiceConfigs);
}
