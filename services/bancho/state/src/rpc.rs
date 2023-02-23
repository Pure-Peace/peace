use crate::{User, UserSessions};
use futures::future::join_all;
use peace_pb::services::bancho_state_rpc::{
    bancho_state_rpc_server::BanchoStateRpc, *,
};
use std::{collections::hash_map::Values, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tools::async_collections::BackgroundService;

const SESSION_NOT_FOUND: &'static str = "session no exists";

#[derive(Debug, Default, Clone)]
pub struct BanchoState {
    pub user_sessions: Arc<RwLock<UserSessions>>,
}

impl BanchoState {
    pub fn start_background_service(&self) {
        let mut session_recycle = BackgroundService::new(|stop| async move {
            println!("start!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
            tokio::select!(
                _ = async {
                    let mut i = 0;

                    loop {
                        i += 1;
                        println!("666");
                        if i > 5 {
                            println!("sm");
                            break;
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                } => {},
                _ = stop.wait_signal() => {}
            );
            println!("end!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")
        });
        session_recycle.start(true).unwrap();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            println!("time to end");
            session_recycle.trigger_signal().unwrap();
        });
    }
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

        info!(target: "session.create", "Session <{session_id}> created");
        Ok(Response::new(CreateUserSessionResponse { session_id }))
    }

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let query = request.into_inner().into();
        self.user_sessions.write().await.delete(&query).await;

        info!(target: "session.delete", "Session <{query:?}> deleted");
        Ok(Response::new(ExecSuccess {}))
    }

    async fn check_user_session_exists(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status> {
        let user = self
            .user_sessions
            .read()
            .await
            .get(&request.into_inner().into())
            .ok_or(Status::not_found(SESSION_NOT_FOUND))?;

        let user_id = {
            let mut user = user.write().await;
            user.update_active();
            user.user_id
        };

        Ok(Response::new(UserSessionExistsResponse { user_id }))
    }

    async fn get_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        let user = self
            .user_sessions
            .read()
            .await
            .get(&request.into_inner().into())
            .ok_or(Status::not_found(SESSION_NOT_FOUND))?;

        let user = user.read().await;

        Ok(Response::new(GetUserSessionResponse {
            session_id: Some(user.session_id.to_owned()),
            user_id: Some(user.user_id),
            username: Some(user.session_id.to_owned()),
            username_unicode: user
                .username_unicode
                .as_ref()
                .map(|s| s.to_owned()),
        }))
    }

    async fn get_user_session_with_fields(
        &self,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        let req = request.into_inner();
        let query = req.query.ok_or(Status::not_found(SESSION_NOT_FOUND))?;

        let user = self
            .user_sessions
            .read()
            .await
            .get(&query.into())
            .ok_or(Status::not_found(SESSION_NOT_FOUND))?;

        let mut res = GetUserSessionResponse::default();
        let fields = UserSessionFields::from(req.fields);

        let user = user.read().await;

        if fields.intersects(UserSessionFields::SessionId) {
            res.session_id = Some(user.session_id.to_owned());
        }

        if fields.intersects(UserSessionFields::UserId) {
            res.user_id = Some(user.user_id);
        }

        if fields.intersects(UserSessionFields::Username) {
            res.username = Some(user.session_id.to_owned());
        }

        if fields.intersects(UserSessionFields::UsernameUnicode) {
            res.username_unicode =
                user.username_unicode.as_ref().map(|s| s.to_owned());
        }

        Ok(Response::new(res))
    }

    async fn get_all_sessions(
        &self,
        _request: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status> {
        let user_sessions = self.user_sessions.read().await;

        async fn collect_data<K>(
            values: Values<'_, K, Arc<RwLock<User>>>,
        ) -> Vec<UserData> {
            join_all(values.map(|u| async {
                let u = u.read().await;
                UserData {
                    session_id: u.session_id.to_owned(),
                    user_id: u.user_id,
                    username: u.username.to_owned(),
                    username_unicode: u.username_unicode.to_owned(),
                    privileges: u.privileges,
                    connection_info: Some(u.connection_info.to_owned()),
                    created_at: u.created_at.to_string(),
                    last_active: u.last_active.to_string(),
                }
            }))
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

        Ok(Response::new(GetAllSessionsResponse {
            len: user_sessions.len() as u64,
            indexed_by_session_id,
            indexed_by_user_id,
            indexed_by_username,
            indexed_by_username_unicode,
        }))
    }
}
