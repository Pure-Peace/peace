use super::{ChatService, DynChatService};
use async_trait::async_trait;
use peace_pb::chat_rpc::chat_rpc_client::ChatRpcClient;
use std::sync::Arc;
use tonic::transport::Channel;

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
        Self::Remote(ChatServiceRemote(client))
    }

    pub fn local() -> Self {
        Self::Local(ChatServiceLocal::new())
    }
}

#[derive(Clone)]
pub struct ChatServiceRemote(ChatRpcClient<Channel>);

#[derive(Clone)]
pub struct ChatServiceLocal {}

impl ChatServiceRemote {
    pub fn client(&self) -> ChatRpcClient<Channel> {
        self.0.clone()
    }
}

impl ChatServiceLocal {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ChatService for ChatServiceImpl {}
