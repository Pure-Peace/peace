use peace_pb::{
    bancho_state::{BanchoPackets, RawUserQuery},
    base::ExecSuccess,
    chat::*,
};
use peace_services::chat::{ChatServiceError, DynChatService};
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
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.chat_service
            .login(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let LogoutRequest { user_query, platforms } = request.into_inner();
        let user_query = user_query
            .ok_or(ChatServiceError::InvalidArgument)?
            .into_user_query()?;

        self.chat_service
            .logout(user_query, platforms.into())
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

    async fn join_channel(
        &self,
        request: Request<JoinChannelRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.chat_service
            .join_channel(request.into_inner())
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }

    async fn leave_channel(
        &self,
        request: Request<LeaveChannelRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.chat_service
            .leave_channel(request.into_inner())
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
            .dequeue_chat_packets(request.into_inner().into_user_query()?)
            .await
            .map_err(|err| err.into())
            .map(Response::new)
    }
}
