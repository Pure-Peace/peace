use peace_pb::chat::*;
use peace_services::chat::DynChatService;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct ChatRpcImpl {
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

    async fn add_user_into_channel(
        &self,
        request: Request<AddUserIntoChannelRequest>,
    ) -> Result<Response<ChannelInfo>, Status> {
        self.chat_service
            .add_user_into_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn remove_user_platforms_from_channel(
        &self,
        request: Request<RemoveUserPlatformsFromChannelRequest>,
    ) -> Result<Response<ChannelInfo>, Status> {
        self.chat_service
            .remove_user_platforms_from_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn remove_user_from_channel(
        &self,
        request: Request<RemoveUserFromChannelRequest>,
    ) -> Result<Response<ChannelInfo>, Status> {
        self.chat_service
            .remove_user_from_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        self.chat_service
            .send_message(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }
}
