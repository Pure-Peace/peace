use super::traits::*;
use crate::{
    bancho_state::BanchoStateError,
    gateway::bancho_endpoints::components::BanchoClientToken, FromRpcClient,
    IntoService, RpcClient,
};
use async_trait::async_trait;
use peace_pb::{
    bancho_state::{bancho_state_rpc_client::BanchoStateRpcClient, *},
    base::ExecSuccess,
};
use std::sync::Arc;
use tonic::{transport::Channel, Code};

#[derive(Debug, Clone)]
pub struct BanchoStateServiceRemote(BanchoStateRpcClient<Channel>);

impl FromRpcClient for BanchoStateServiceRemote {
    #[inline]
    fn from_client(client: Self::Client) -> Self {
        Self(client)
    }
}

impl RpcClient for BanchoStateServiceRemote {
    type Client = BanchoStateRpcClient<Channel>;

    fn client(&self) -> Self::Client {
        self.0.clone()
    }
}

impl IntoService<DynBanchoStateService> for BanchoStateServiceRemote {
    #[inline]
    fn into_service(self) -> DynBanchoStateService {
        Arc::new(self) as DynBanchoStateService
    }
}

#[async_trait]
impl BanchoStateService for BanchoStateServiceRemote {}

#[async_trait]
impl BroadcastBanchoPackets for BanchoStateServiceRemote {
    async fn broadcast_bancho_packets(
        &self,
        request: BroadcastBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .broadcast_bancho_packets(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl EnqueueBanchoPackets for BanchoStateServiceRemote {
    async fn enqueue_bancho_packets(
        &self,
        request: EnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .enqueue_bancho_packets(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl BatchEnqueueBanchoPackets for BanchoStateServiceRemote {
    async fn batch_enqueue_bancho_packets(
        &self,
        request: BatchEnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .batch_enqueue_bancho_packets(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl DequeueBanchoPackets for BanchoStateServiceRemote {
    async fn dequeue_bancho_packets(
        &self,
        request: DequeueBanchoPacketsRequest,
    ) -> Result<BanchoPackets, BanchoStateError> {
        self.client()
            .dequeue_bancho_packets(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl CreateUserSession for BanchoStateServiceRemote {
    async fn create_user_session(
        &self,
        request: CreateUserSessionRequest,
    ) -> Result<CreateUserSessionResponse, BanchoStateError> {
        self.client()
            .create_user_session(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl DeleteUserSession for BanchoStateServiceRemote {
    async fn delete_user_session(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .delete_user_session(Into::<RawUserQuery>::into(query))
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl CheckUserToken for BanchoStateServiceRemote {
    async fn check_user_token(
        &self,
        token: BanchoClientToken,
    ) -> Result<CheckUserTokenResponse, BanchoStateError> {
        self.client()
            .check_user_token(token)
            .await
            .map_err(|status| {
                if status.code() == Code::NotFound {
                    BanchoStateError::SessionNotExists
                } else {
                    BanchoStateError::RpcError(status)
                }
            })
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl IsUserOnline for BanchoStateServiceRemote {
    async fn is_user_online(
        &self,
        query: UserQuery,
    ) -> Result<UserOnlineResponse, BanchoStateError> {
        self.client()
            .is_user_online(Into::<RawUserQuery>::into(query))
            .await
            .map_err(|status| {
                if status.code() == Code::NotFound {
                    BanchoStateError::SessionNotExists
                } else {
                    BanchoStateError::RpcError(status)
                }
            })
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl GetUserSession for BanchoStateServiceRemote {
    async fn get_user_session(
        &self,
        query: UserQuery,
    ) -> Result<GetUserSessionResponse, BanchoStateError> {
        self.client()
            .get_user_session(Into::<RawUserQuery>::into(query))
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl GetUserSessionWithFields for BanchoStateServiceRemote {
    async fn get_user_session_with_fields(
        &self,
        request: RawUserQueryWithFields,
    ) -> Result<GetUserSessionResponse, BanchoStateError> {
        self.client()
            .get_user_session_with_fields(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl GetAllSessions for BanchoStateServiceRemote {
    async fn get_all_sessions(
        &self,
    ) -> Result<GetAllSessionsResponse, BanchoStateError> {
        self.client()
            .get_all_sessions(GetAllSessionsRequest {})
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl SendUserStatsPacket for BanchoStateServiceRemote {
    async fn send_user_stats_packet(
        &self,
        request: SendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .send_user_stats_packet(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl BatchSendUserStatsPacket for BanchoStateServiceRemote {
    async fn batch_send_user_stats_packet(
        &self,
        request: BatchSendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .batch_send_user_stats_packet(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl SendAllPresences for BanchoStateServiceRemote {
    async fn send_all_presences(
        &self,
        request: SendAllPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .send_all_presences(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl BatchSendPresences for BanchoStateServiceRemote {
    async fn batch_send_presences(
        &self,
        request: BatchSendPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .batch_send_presences(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl UpdatePresenceFilter for BanchoStateServiceRemote {
    async fn update_presence_filter(
        &self,
        request: UpdatePresenceFilterRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .update_presence_filter(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}

#[async_trait]
impl UpdateUserBanchoStatus for BanchoStateServiceRemote {
    async fn update_user_bancho_status(
        &self,
        request: UpdateUserBanchoStatusRequest,
    ) -> Result<ExecSuccess, BanchoStateError> {
        self.client()
            .update_user_bancho_status(request)
            .await
            .map_err(BanchoStateError::RpcError)
            .map(|resp| resp.into_inner())
    }
}
