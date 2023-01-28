use crate::{User, UserSessions};
use futures::future::join_all;
use peace_pb::services::bancho_state_rpc::{
    bancho_state_rpc_server::BanchoStateRpc,
    show_all_sessions_response::UserData, *,
};
use std::{collections::hash_map::Values, sync::Arc};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct BanchoState {
    pub user_sessions: Arc<RwLock<UserSessions>>,
}

#[tonic::async_trait]
impl BanchoStateRpc for BanchoState {
    async fn broadcast_bancho_packets(
        &self,
        request: Request<BroadcastBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        unimplemented!()
    }

    async fn enqueue_bancho_packets(
        &self,
        request: Request<EnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        unimplemented!()
    }

    async fn batch_enqueue_bancho_packets(
        &self,
        request: Request<BatchEnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        unimplemented!()
    }

    async fn dequeue_bancho_packets(
        &self,
        request: Request<DequeueBanchoPacketsRequest>,
    ) -> Result<Response<BanchoPackets>, Status> {
        unimplemented!()
    }

    async fn batch_dequeue_bancho_packets(
        &self,
        request: Request<BatchDequeueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        unimplemented!()
    }

    async fn create_user_session(
        &self,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status> {
        let session_id = self
            .user_sessions
            .write()
            .await
            .create(request.into_inner().into())
            .await;

        Ok(Response::new(CreateUserSessionResponse { session_id }))
    }

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        self.user_sessions
            .write()
            .await
            .delete(&request.into_inner().into())
            .await;

        Ok(Response::new(ExecSuccess {}))
    }

    async fn is_user_session_exists(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<IsUserSessionExistsResponse>, Status> {
        Ok(Response::new(IsUserSessionExistsResponse {
            session_id: {
                if let Some(user) = self
                    .user_sessions
                    .read()
                    .await
                    .get(&request.into_inner().into())
                {
                    Some(user.read().await.session_id.to_owned())
                } else {
                    None
                }
            },
        }))
    }

    async fn show_all_sessions(
        &self,
        _request: Request<ShowAllSessionsRequest>,
    ) -> Result<Response<ShowAllSessionsResponse>, Status> {
        let user_sessions = self.user_sessions.read().await;

        async fn collect_data<K>(
            values: Values<'_, K, Arc<RwLock<User>>>,
        ) -> Vec<UserData> {
            join_all(values.map(|u| async { u.read().await.to_owned().into() }))
                .await
        }

        let indexed_by_session_id =
            collect_data(user_sessions.indexed_by_session_id.values()).await;
        let indexed_by_user_id =
            collect_data(user_sessions.indexed_by_user_id.values()).await;
        let indexed_by_username =
            collect_data(user_sessions.indexed_by_username.values()).await;
        let indexed_by_username_unicode =
            collect_data(user_sessions.indexed_by_username_unicode.values())
                .await;

        Ok(Response::new(ShowAllSessionsResponse {
            len: user_sessions.len() as u64,
            indexed_by_session_id,
            indexed_by_user_id,
            indexed_by_username,
            indexed_by_username_unicode,
        }))
    }
}
