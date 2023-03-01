use super::ChatService;
use async_trait::async_trait;
use peace_pb::chat_rpc::{chat_rpc_client::ChatRpcClient, *};
use tonic::transport::Channel;

#[derive(Clone)]
pub enum ChatServiceImpl {
    Remote(ChatServiceRemote),
    Local(ChatServiceLocal),
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
