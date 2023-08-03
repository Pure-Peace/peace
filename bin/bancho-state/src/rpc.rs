use core_bancho_state::DynBanchoStateService;
use peace_domain::bancho::BanchoClientToken;
use peace_pb::{bancho_state::*, base::ExecSuccess};
use peace_unique_id::Ulid;
use std::str::FromStr;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct BanchoStateRpcImpl {
    pub bancho_state_service: DynBanchoStateService,
}

impl BanchoStateRpcImpl {
    pub fn new(bancho_state_service: DynBanchoStateService) -> Self {
        Self { bancho_state_service }
    }
}

#[tonic::async_trait]
impl bancho_state_rpc_server::BanchoStateRpc for BanchoStateRpcImpl {
    async fn broadcast_bancho_packets(
        &self,
        request: Request<BroadcastBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .broadcast_bancho_packets(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn enqueue_bancho_packets(
        &self,
        request: Request<EnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .enqueue_bancho_packets(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn batch_enqueue_bancho_packets(
        &self,
        request: Request<BatchEnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .batch_enqueue_bancho_packets(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn dequeue_bancho_packets(
        &self,
        request: Request<DequeueBanchoPacketsRequest>,
    ) -> Result<Response<BanchoPackets>, Status> {
        let res = self
            .bancho_state_service
            .dequeue_bancho_packets(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn create_user_session(
        &self,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status> {
        let res = self
            .bancho_state_service
            .create_user_session(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .delete_user_session(request.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }

    async fn check_user_token(
        &self,
        request: Request<CheckUserTokenRequest>,
    ) -> Result<Response<CheckUserTokenResponse>, Status> {
        let CheckUserTokenRequest { user_id, session_id, signature } =
            request.into_inner();

        let client_token = BanchoClientToken {
            user_id,
            session_id: Ulid::from_str(session_id.as_str())
                .map_err(|err| Status::invalid_argument(err.to_string()))?,
            signature,
        };

        let res =
            self.bancho_state_service.check_user_token(client_token).await?;

        Ok(Response::new(res))
    }

    async fn is_user_online(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserOnlineResponse>, Status> {
        let res = self
            .bancho_state_service
            .is_user_online(request.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }

    async fn get_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        let res = self
            .bancho_state_service
            .get_user_session(request.into_inner().into_user_query()?)
            .await?;

        Ok(Response::new(res))
    }

    async fn get_user_session_with_fields(
        &self,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        let res = self
            .bancho_state_service
            .get_user_session_with_fields(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn get_all_sessions(
        &self,
        _: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status> {
        let res = self.bancho_state_service.get_all_sessions().await?;

        Ok(Response::new(res))
    }

    async fn send_user_stats_packet(
        &self,
        request: Request<SendUserStatsPacketRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .send_user_stats_packet(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn send_all_presences(
        &self,
        request: Request<SendAllPresencesRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .send_all_presences(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn batch_send_user_stats_packet(
        &self,
        request: Request<BatchSendUserStatsPacketRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .batch_send_user_stats_packet(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn update_presence_filter(
        &self,
        request: Request<UpdatePresenceFilterRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .update_presence_filter(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn update_user_bancho_status(
        &self,
        request: Request<UpdateUserBanchoStatusRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .update_user_bancho_status(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }

    async fn batch_send_presences(
        &self,
        request: Request<BatchSendPresencesRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let res = self
            .bancho_state_service
            .batch_send_presences(request.into_inner())
            .await?;

        Ok(Response::new(res))
    }
}
