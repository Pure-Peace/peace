use peace_pb::{
    bancho_state::{BanchoPackets, RawUserQuery},
    base::ExecSuccess,
    chat::*,
};
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
    async fn create_queue(
        &self,
        request: Request<CreateQueueRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.chat_service
            .create_queue(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn remove_queue(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.chat_service
            .remove_queue(request.into_inner().into_user_query()?)
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

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
    ) -> Result<Response<ExecSuccess>, Status> {
        self.chat_service
            .add_user_into_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn remove_user_platforms_from_channel(
        &self,
        request: Request<RemoveUserPlatformsFromChannelRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.chat_service
            .remove_user_platforms_from_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn remove_user_from_channel(
        &self,
        request: Request<RemoveUserFromChannelRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
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

    async fn pull_chat_packets(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<BanchoPackets>, Status> {
        self.chat_service
            .pull_chat_packets(request.into_inner().into_user_query()?)
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }
}
