use peace_pb::chat::*;

#[derive(Debug, Default, Clone)]
pub struct ChatRpcImpl {}

#[tonic::async_trait]
impl chat_rpc_server::ChatRpc for ChatRpcImpl {}
