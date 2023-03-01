use crate::Chat;
use peace_pb::chat_rpc::*;

#[tonic::async_trait]
impl chat_rpc_server::ChatRpc for Chat {}
