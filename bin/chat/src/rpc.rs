use peace_pb::chat::*;
use peace_services::chat::DynChatService;

#[derive(Clone)]
pub struct ChatRpcImpl {
    #[allow(dead_code)]
    chat_service: DynChatService,
}

impl ChatRpcImpl {
    pub fn new(chat_service: DynChatService) -> Self {
        Self { chat_service }
    }
}

#[tonic::async_trait]
impl chat_rpc_server::ChatRpc for ChatRpcImpl {}
