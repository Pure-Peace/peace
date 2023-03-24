use peace_pb::chat::*;
use peace_services::chat::DynChatService;
use tonic::{Request, Response, Status};

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
impl chat_rpc_server::ChatRpc for ChatRpcImpl {
    async fn get_public_channels(
        &self,
        _: Request<GetPublicChannelsRequest>,
    ) -> Result<Response<GetPublicChannelsResponse>, Status> {
        self.chat_service
            .get_public_channels()
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn join_into_channel(
        &self,
        request: Request<JoinIntoChannelRequest>,
    ) -> Result<Response<ChannelInfo>, Status> {
        self.chat_service
            .join_into_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn leave_from_channel(
        &self,
        request: Request<LeaveFromChannelRequest>,
    ) -> Result<Response<ChannelInfo>, Status> {
        self.chat_service
            .leave_from_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn delete_from_channel(
        &self,
        request: Request<DeleteFromChannelRequest>,
    ) -> Result<Response<ChannelInfo>, Status> {
        self.chat_service
            .delete_from_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }
}
