use peace_pb::chat_rpc::chat_rpc_server::ChatRpc;

#[derive(Debug, Default, Clone)]
pub struct ChatService {}

#[tonic::async_trait]
impl ChatRpc for ChatService {}
