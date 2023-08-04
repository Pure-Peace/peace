use crate::*;
use bancho_packets::Packet;
use domain_users::PasswordError;
use pb_bancho::*;
use pb_bancho_state::UserQuery;
use std::{net::IpAddr, sync::Arc};
use tonic::async_trait;
use tools::async_collections::{
    BackgroundTask, BackgroundTaskError, CommonRecycleBackgroundTaskConfig,
};

pub type DynBanchoService = Arc<dyn BanchoService + Send + Sync>;
pub type DynBanchoBackgroundService =
    Arc<dyn BanchoBackgroundService + Send + Sync>;
pub type DynPasswordService = Arc<dyn PasswordService + Send + Sync>;

#[async_trait]
pub trait PasswordBackgroundService {
    fn start_password_caches_recycle(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    );
    fn stop_password_caches_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError>;
}

pub trait GetPasswordCacheStore {
    fn cache_store(&self) -> &PasswordCacheStore;
}

#[async_trait]
pub trait PasswordService {
    async fn verify_password(
        &self,
        hashed_password: &str,
        password: &str,
    ) -> Result<(), PasswordError>;
}

#[async_trait]
pub trait BanchoBackgroundService: PasswordBackgroundService {
    fn start_all(&self, configs: BanchoBackgroundServiceConfigs);
}

pub trait BanchoService:
    Login
    + BatchProcessPackets
    + ProcessPackets
    + ClientPing
    + RequestStatusUpdate
    + PresenceRequestAll
    + RequestStats
    + ChangeAction
    + ReceiveUpdates
    + ToggleBlockNonFriendDms
    + UserLogout
    + RequestPresence
    + SpectateStop
    + SpectateCant
    + LobbyPart
    + LobbyJoin
{
}

#[async_trait]
pub trait Login {
    async fn login(
        &self,
        client_ip: IpAddr,
        request: LoginRequest,
    ) -> Result<LoginSuccess, BanchoServiceError>;
}

#[async_trait]
pub trait BatchProcessPackets {
    async fn batch_process_bancho_packets(
        &self,
        request: BatchProcessBanchoPacketsRequest,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessPackets {
    async fn process_bancho_packet(
        &self,
        user_id: i32,
        packet: Packet<'_>,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ClientPing {
    async fn ping(&self) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait RequestStatusUpdate {
    async fn request_status_update(
        &self,
        user_query: UserQuery,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait PresenceRequestAll {
    async fn presence_request_all(
        &self,
        user_query: UserQuery,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait RequestStats {
    async fn request_stats(
        &self,
        request: StatsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait ChangeAction {
    async fn change_action(
        &self,
        request: ChangeActionRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait ReceiveUpdates {
    async fn receive_updates(
        &self,
        request: ReceiveUpdatesRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait ToggleBlockNonFriendDms {
    async fn toggle_block_non_friend_dms(
        &self,
        request: ToggleBlockNonFriendDmsRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait UserLogout {
    async fn user_logout(
        &self,
        user_query: UserQuery,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait RequestPresence {
    async fn request_presence(
        &self,
        request: PresenceRequest,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait SpectateStop {
    async fn spectate_stop(
        &self,
        user_query: UserQuery,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait SpectateCant {
    async fn spectate_cant(
        &self,
        user_query: UserQuery,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait LobbyPart {
    async fn lobby_part(
        &self,
        user_query: UserQuery,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

#[async_trait]
pub trait LobbyJoin {
    async fn lobby_join(
        &self,
        user_query: UserQuery,
    ) -> Result<HandleCompleted, BanchoServiceError>;
}

pub trait BanchoPacketProcessor:
    ProcessSendPublicMessage
    + ProcessSendPrivateMessage
    + ProcessUserChannelJoin
    + ProcessUserChannelPart
    + ProcessUserRequestStatusUpdate
    + ProcessUserPresenceRequestAll
    + ProcessUserStatsRequest
    + ProcessUserChangeAction
    + ProcessUserReceiveUpdates
    + ProcessUserToggleBlockNonFriendDms
    + ProcessUserLogout
    + ProcessUserPresenceRequest
{
}

#[async_trait]
pub trait ProcessSendPublicMessage {
    async fn send_public_message(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessSendPrivateMessage {
    async fn send_private_message(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserChannelJoin {
    async fn user_channel_join(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserChannelPart {
    async fn user_channel_part(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserRequestStatusUpdate {
    async fn user_request_status_update(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserPresenceRequestAll {
    async fn user_presence_request_all(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserStatsRequest {
    async fn user_stats_request(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserChangeAction {
    async fn user_change_action(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserReceiveUpdates {
    async fn user_receive_updates(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserToggleBlockNonFriendDms {
    async fn user_toggle_block_non_friend_dms(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserLogout {
    async fn user_logout(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}

#[async_trait]
pub trait ProcessUserPresenceRequest {
    async fn user_presence_request(
        &self,
    ) -> Result<HandleCompleted, ProcessBanchoPacketError>;
}
