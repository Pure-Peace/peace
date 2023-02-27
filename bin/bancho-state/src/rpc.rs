use crate::BanchoState;
use peace_pb::services::bancho_state_rpc::*;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl bancho_state_rpc_server::BanchoStateRpc for BanchoState {
    async fn broadcast_bancho_packets(
        &self,
        request: Request<BroadcastBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.packets_repository.broadcast_bancho_packets(request).await
    }

    async fn enqueue_bancho_packets(
        &self,
        request: Request<EnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.packets_repository.enqueue_bancho_packets(request).await
    }

    async fn batch_enqueue_bancho_packets(
        &self,
        request: Request<BatchEnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.packets_repository.batch_enqueue_bancho_packets(request).await
    }

    async fn dequeue_bancho_packets(
        &self,
        request: Request<DequeueBanchoPacketsRequest>,
    ) -> Result<Response<BanchoPackets>, Status> {
        self.packets_repository.dequeue_bancho_packets(request).await
    }

    async fn batch_dequeue_bancho_packets(
        &self,
        request: Request<BatchDequeueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.packets_repository.batch_dequeue_bancho_packets(request).await
    }

    async fn create_user_session(
        &self,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status> {
        self.sessions_repository
            .create_user_session(
                self.app_state_repository.user_sessions(),
                request,
            )
            .await
    }

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.sessions_repository
            .delete_user_session(
                self.app_state_repository.user_sessions(),
                request,
            )
            .await
    }

    async fn check_user_session_exists(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status> {
        self.sessions_repository
            .check_user_session_exists(
                self.app_state_repository.user_sessions(),
                request,
            )
            .await
    }

    async fn get_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        self.sessions_repository
            .get_user_session(
                self.app_state_repository.user_sessions(),
                request,
            )
            .await
    }

    async fn get_user_session_with_fields(
        &self,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        self.sessions_repository
            .get_user_session_with_fields(
                self.app_state_repository.user_sessions(),
                request,
            )
            .await
    }

    async fn get_all_sessions(
        &self,
        _request: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status> {
        self.sessions_repository
            .get_all_sessions(
                self.app_state_repository.user_sessions(),
                _request,
            )
            .await
    }
}
