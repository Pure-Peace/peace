use super::BanchoStateService;
use crate::bancho_state::{DynBanchoStateService, User, UserSessions};
use async_trait::async_trait;
use peace_pb::bancho_state_rpc::{
    bancho_state_rpc_client::BanchoStateRpcClient, *,
};
use std::{collections::hash_map::Values, sync::Arc};
use tokio::sync::RwLock;
use tonic::{transport::Channel, Request, Response, Status};

pub const SESSION_NOT_FOUND: &'static str = "session no exists";

#[derive(Debug, Clone)]
pub enum BanchoStateServiceImpl {
    Remote(BanchoStateServiceRemote),
    Local(BanchoStateServiceLocal),
}

impl BanchoStateServiceImpl {
    pub fn into_service(self) -> DynBanchoStateService {
        Arc::new(self) as DynBanchoStateService
    }

    pub fn remote(client: BanchoStateRpcClient<Channel>) -> Self {
        Self::Remote(BanchoStateServiceRemote(client))
    }

    pub fn local(user_sessions: Arc<RwLock<UserSessions>>) -> Self {
        Self::Local(BanchoStateServiceLocal::new(user_sessions))
    }
}

#[derive(Debug, Clone)]
pub struct BanchoStateServiceRemote(BanchoStateRpcClient<Channel>);

impl BanchoStateServiceRemote {
    pub fn new(client: BanchoStateRpcClient<Channel>) -> Self {
        Self(client)
    }

    pub fn client(&self) -> BanchoStateRpcClient<Channel> {
        self.0.clone()
    }
}

#[derive(Debug, Default, Clone)]
pub struct BanchoStateServiceLocal {
    user_sessions: Arc<RwLock<UserSessions>>,
}

impl BanchoStateServiceLocal {
    pub fn new(user_sessions: Arc<RwLock<UserSessions>>) -> Self {
        Self { user_sessions }
    }
}

#[async_trait]
impl BanchoStateService for BanchoStateServiceImpl {
    async fn broadcast_bancho_packets(
        &self,
        request: Request<BroadcastBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().broadcast_bancho_packets(request).await
            },
            BanchoStateServiceImpl::Local(svc) => unimplemented!(),
        }
    }

    async fn enqueue_bancho_packets(
        &self,
        request: Request<EnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().enqueue_bancho_packets(request).await
            },
            BanchoStateServiceImpl::Local(svc) => unimplemented!(),
        }
    }

    async fn batch_enqueue_bancho_packets(
        &self,
        request: Request<BatchEnqueueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().batch_enqueue_bancho_packets(request).await
            },
            BanchoStateServiceImpl::Local(svc) => unimplemented!(),
        }
    }

    async fn dequeue_bancho_packets(
        &self,
        request: Request<DequeueBanchoPacketsRequest>,
    ) -> Result<Response<BanchoPackets>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().dequeue_bancho_packets(request).await
            },
            BanchoStateServiceImpl::Local(svc) => unimplemented!(),
        }
    }

    async fn batch_dequeue_bancho_packets(
        &self,
        request: Request<BatchDequeueBanchoPacketsRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().batch_dequeue_bancho_packets(request).await
            },
            BanchoStateServiceImpl::Local(svc) => unimplemented!(),
        }
    }

    async fn create_user_session(
        &self,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().create_user_session(request).await
            },
            BanchoStateServiceImpl::Local(svc) => {
                // Create a new user session using the provided request.
                let session_id = svc
                    .user_sessions
                    .write()
                    .await
                    .create(request.into_inner().into())
                    .await;

                // Log the session creation.
                info!(target: "session.create", "Session <{session_id}> created");

                // Return the new session ID in a response.
                Ok(Response::new(CreateUserSessionResponse { session_id }))
            },
        }
    }

    async fn delete_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().delete_user_session(request).await
            },
            BanchoStateServiceImpl::Local(svc) => {
                let query = request.into_inner().into();

                // Delete the session using the query.
                svc.user_sessions.write().await.delete(&query).await;

                // Log that the session was deleted.
                info!(target: "session.delete", "Session <{query:?}> deleted");

                // Return a success message.
                Ok(Response::new(ExecSuccess {}))
            },
        }
    }

    async fn check_user_session_exists(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().check_user_session_exists(request).await
            },
            BanchoStateServiceImpl::Local(svc) => {
                // Retrieve the user session from the user session store.
                let user = svc
                    .user_sessions
                    .read()
                    .await
                    .get(&request.into_inner().into())
                    .ok_or(Status::not_found(SESSION_NOT_FOUND))?;

                // Update the user's last active time and retrieve their ID.
                let user_id = {
                    let mut user = user.write().await;
                    user.update_active();
                    user.user_id
                };

                // Return the user ID in a response.
                Ok(Response::new(UserSessionExistsResponse { user_id }))
            },
        }
    }

    async fn get_user_session(
        &self,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().get_user_session(request).await
            },
            BanchoStateServiceImpl::Local(svc) => {
                // Get the user session based on the provided query
                let user = svc
                    .user_sessions
                    .read()
                    .await
                    .get(&request.into_inner().into())
                    .ok_or(Status::not_found(SESSION_NOT_FOUND))?;

                // Get a read lock on the user session data
                let user = user.read().await;

                // Create a response with the user session details
                Ok(Response::new(GetUserSessionResponse {
                    // Copy the session ID into the response
                    session_id: Some(user.session_id.to_owned()),
                    // Copy the user ID into the response
                    user_id: Some(user.user_id),
                    // Copy the username into the response
                    username: Some(user.session_id.to_owned()),
                    // Copy the Unicode username into the response, if it exists
                    username_unicode: user
                        .username_unicode
                        .as_ref()
                        .map(|s| s.to_owned()),
                }))
            },
        }
    }

    async fn get_user_session_with_fields(
        &self,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().get_user_session_with_fields(request).await
            },
            BanchoStateServiceImpl::Local(svc) => {
                // Extract the query and fields from the request
                let req = request.into_inner();
                let query =
                    req.query.ok_or(Status::not_found(SESSION_NOT_FOUND))?;

                // Retrieve the user session from the database
                let user = svc
                    .user_sessions
                    .read()
                    .await
                    .get(&query.into())
                    .ok_or(Status::not_found(SESSION_NOT_FOUND))?;

                // Initialize the response and extract the requested fields
                let mut res = GetUserSessionResponse::default();
                let fields = UserSessionFields::from(req.fields);

                // Read the user session data from the database
                let user = user.read().await;

                // Set the response fields based on the requested fields
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

                // Return the response
                Ok(Response::new(res))
            },
        }
    }

    async fn get_all_sessions(
        &self,
        _request: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status> {
        match self {
            BanchoStateServiceImpl::Remote(svc) => {
                svc.client().get_all_sessions(_request).await
            },
            BanchoStateServiceImpl::Local(svc) => {
                // Get a read lock on the `user_sessions` hash map
                let user_sessions = svc.user_sessions.read().await;

                // Define a helper function to collect data from the hash map
                async fn collect_data<K>(
                    values: Values<'_, K, Arc<RwLock<User>>>,
                ) -> Vec<UserData> {
                    // Use `join_all` to asynchronously process all elements in
                    // the `values` iterator
                    futures::future::join_all(values.map(|u| async {
                        // Get a read lock on the user object
                        let u = u.read().await;

                        // Create a `UserData` object with the user's session
                        // data
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

                // Collect session data by index
                let indexed_by_session_id =
                    collect_data(user_sessions.indexed_by_session_id.values())
                        .await;
                let indexed_by_user_id =
                    collect_data(user_sessions.indexed_by_user_id.values())
                        .await;
                let indexed_by_username =
                    collect_data(user_sessions.indexed_by_username.values())
                        .await;
                let indexed_by_username_unicode = collect_data(
                    user_sessions.indexed_by_username_unicode.values(),
                )
                .await;

                // Return a `GetAllSessionsResponse` message containing the
                // session data
                Ok(Response::new(GetAllSessionsResponse {
                    len: user_sessions.len() as u64,
                    indexed_by_session_id,
                    indexed_by_user_id,
                    indexed_by_username,
                    indexed_by_username_unicode,
                }))
            },
        }
    }
}