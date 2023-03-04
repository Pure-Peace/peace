use crate::bancho::BanchoServiceError;
use peace_domain::users::PasswordError;
use peace_pb::bancho_rpc::*;
use std::{net::IpAddr, sync::Arc};
use tonic::async_trait;
use tools::async_collections::{BackgroundTask, BackgroundTaskError};

pub type DynBanchoService = Arc<dyn BanchoService + Send + Sync>;
pub type DynBanchoBackgroundService =
    Arc<dyn BanchoBackgroundService + Send + Sync>;
pub type DynPasswordService = Arc<dyn PasswordService + Send + Sync>;

#[async_trait]
pub trait BanchoBackgroundService {
    fn start_all(&self);
    fn start_password_caches_recycle(&self);
    fn stop_password_caches_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError>;
}

#[async_trait]
pub trait BanchoService {
    async fn ping(
        &self,
        request: PingRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;

    async fn login(
        &self,
        client_ip: IpAddr,
        request: LoginRequest,
    ) -> Result<LoginSuccess, BanchoServiceError>;

    async fn request_status_update(
        &self,
        request: RequestStatusUpdateRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;

    async fn presence_request_all(
        &self,
        request: PresenceRequestAllRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;

    async fn spectate_stop(
        &self,
        request: SpectateStopRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;

    async fn spectate_cant(
        &self,
        request: SpectateCantRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;

    async fn lobby_part(
        &self,
        request: LobbyPartRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;

    async fn lobby_join(
        &self,
        request: LobbyJoinRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait PasswordService {
    async fn verify_password(
        &self,
        hashed_password: &str,
        password: &str,
    ) -> Result<(), PasswordError>;
}
