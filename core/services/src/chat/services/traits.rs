use crate::{bancho_state::BanchoMessageQueue, chat::*, DumpData, DumpToDisk};
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

pub trait ChannelStore {
    fn channels(&self) -> &Arc<Channels> {
        unimplemented!()
    }
}

pub trait UserSessionsStore {
    fn user_sessions(&self) -> &Arc<UserSessions> {
        unimplemented!()
    }
}

pub trait NotifyMessagesQueue {
    fn notify_queue(&self) -> &Arc<BanchoMessageQueue> {
        unimplemented!()
    }
}

#[async_trait]
pub trait ChatService:
    UserSessionsStore
    + NotifyMessagesQueue
    + ChannelStore
    + DumpData<ChatServiceDump>
    + DumpToDisk<ChatServiceDump>
{
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
