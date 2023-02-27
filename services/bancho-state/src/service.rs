use crate::{repositorys::*, User, UserSessions};
use async_trait::async_trait;
use peace_pb::services::bancho_state_rpc::*;
use std::{collections::hash_map::Values, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tools::async_collections::BackgroundService;

pub const SESSION_NOT_FOUND: &'static str = "session no exists";

#[derive(Debug, Default, Clone)]
pub struct AppState {
    /// The collection of user sessions currently active on the server.
    pub user_sessions: Arc<RwLock<UserSessions>>,
}

impl AppStateRepository for AppState {
    fn user_sessions(&self) -> Arc<RwLock<UserSessions>> {
        self.user_sessions.clone()
    }
}

#[derive(Debug, Default, Clone)]
pub struct BanchoStatePacketsRepository;

#[derive(Debug, Default, Clone)]
pub struct BanchoStateBackgroundServiceRepository;

#[derive(Debug, Default, Clone)]
pub struct BanchoStateSessionsRepository;

#[async_trait]
impl BackgroundServiceRepository for BanchoStateBackgroundServiceRepository {
    fn start_background_service(&self) {
        let mut session_recycle = BackgroundService::new(|stop| async move {
            // Start the service
            println!("Starting session recycling service...");

            tokio::select!(
                _ = async {
                    let mut i = 0;

                    loop {
                        i += 1;
                        println!("Session recycling service running... iteration {}", i);

                        if i > 5 {
                            println!("Session recycling service stopped.");
                            break;
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                } => {},
                _ = stop.wait_signal() => {}
            );

            // End the service
            println!("Session recycling service stopped.");
        });

        // Start the session recycling service
        session_recycle.start(true).unwrap();

        // Schedule the service to stop after 10 seconds
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            session_recycle.trigger_signal().unwrap();
        });
    }
}

#[async_trait]
impl PacketsRepository for BanchoStatePacketsRepository {
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
}

#[async_trait]
impl SessionsRepository for BanchoStateSessionsRepository {
    async fn create_user_session(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<CreateUserSessionRequest>,
    ) -> Result<Response<CreateUserSessionResponse>, Status> {
        // Create a new user session using the provided request.
        let session_id = user_sessions
            .write()
            .await
            .create(request.into_inner().into())
            .await;

        // Log the session creation.
        info!(target: "session.create", "Session <{session_id}> created");

        // Return the new session ID in a response.
        Ok(Response::new(CreateUserSessionResponse { session_id }))
    }

    async fn delete_user_session(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQuery>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let query = request.into_inner().into();

        // Delete the session using the query.
        user_sessions.write().await.delete(&query).await;

        // Log that the session was deleted.
        info!(target: "session.delete", "Session <{query:?}> deleted");

        // Return a success message.
        Ok(Response::new(ExecSuccess {}))
    }

    async fn check_user_session_exists(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQuery>,
    ) -> Result<Response<UserSessionExistsResponse>, Status> {
        // Retrieve the user session from the user session store.
        let user = user_sessions
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
    }

    async fn get_user_session(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQuery>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        // Get the user session based on the provided query
        let user = user_sessions
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
    }

    async fn get_user_session_with_fields(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        request: Request<RawUserQueryWithFields>,
    ) -> Result<Response<GetUserSessionResponse>, Status> {
        // Extract the query and fields from the request
        let req = request.into_inner();
        let query = req.query.ok_or(Status::not_found(SESSION_NOT_FOUND))?;

        // Retrieve the user session from the database
        let user = user_sessions
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
    }

    async fn get_all_sessions(
        &self,
        user_sessions: Arc<RwLock<UserSessions>>,
        _request: Request<GetAllSessionsRequest>,
    ) -> Result<Response<GetAllSessionsResponse>, Status> {
        // Get a read lock on the `user_sessions` hash map
        let user_sessions = user_sessions.read().await;

        // Define a helper function to collect data from the hash map
        async fn collect_data<K>(
            values: Values<'_, K, Arc<RwLock<User>>>,
        ) -> Vec<UserData> {
            // Use `join_all` to asynchronously process all elements in the `values` iterator
            futures::future::join_all(values.map(|u| async {
                // Get a read lock on the user object
                let u = u.read().await;

                // Create a `UserData` object with the user's session data
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
            collect_data(user_sessions.indexed_by_session_id.values()).await;
        let indexed_by_user_id =
            collect_data(user_sessions.indexed_by_user_id.values()).await;
        let indexed_by_username =
            collect_data(user_sessions.indexed_by_username.values()).await;
        let indexed_by_username_unicode =
            collect_data(user_sessions.indexed_by_username_unicode.values())
                .await;

        // Return a `GetAllSessionsResponse` message containing the session data
        Ok(Response::new(GetAllSessionsResponse {
            len: user_sessions.len() as u64,
            indexed_by_session_id,
            indexed_by_user_id,
            indexed_by_username,
            indexed_by_username_unicode,
        }))
    }
}
