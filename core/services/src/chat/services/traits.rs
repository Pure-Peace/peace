use crate::chat::*;
use peace_pb::chat::ChannelQuery;
use std::sync::Arc;
use tonic::async_trait;

pub type DynChatService = Arc<dyn ChatService + Send + Sync>;
pub type DynChannelService = Arc<dyn ChannelService + Send + Sync>;

#[async_trait]
pub trait ChatService {}

#[async_trait]
pub trait ChannelService {
    fn channels(&self) -> &Arc<Channels>;

    async fn initialize_public_channels(&self);

    async fn create(
        &self,
        id: u32,
        name: String,
        channel_type: ChannelType,
        description: Option<String>,
        users: Vec<i32>,
    ) -> Arc<Channel>;

    async fn delete(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn get(&self, query: &ChannelQuery) -> Option<Arc<Channel>>;

    async fn exists(&self, query: &ChannelQuery) -> bool;

    async fn clear(&self);

    fn len(&self) -> usize;
}
