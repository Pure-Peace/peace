use super::BanchoStateBackgroundServiceConfigs;
use crate::{
    bancho_state::{BanchoStateError, Packet, Session, UserSessions},
    gateway::bancho_endpoints::components::BanchoClientToken,
};
use async_trait::async_trait;
use peace_domain::bancho_state::CreateSessionDto;
use peace_pb::{bancho_state::*, base::ExecSuccess};
use std::sync::Arc;
use tokio::sync::Mutex;
use tools::{
    async_collections::{
        BackgroundTask, BackgroundTaskError, CommonRecycleBackgroundTaskConfig,
        LoopBackgroundTaskConfig,
    },
    message_queue::MessageQueue,
    Ulid,
};

pub type DynBanchoStateService = Arc<dyn BanchoStateService + Send + Sync>;
pub type DynBanchoStateBackgroundService =
    Arc<dyn BanchoStateBackgroundService + Send + Sync>;
pub type DynUserSessionsService = Arc<dyn UserSessionsService + Send + Sync>;

#[async_trait]
pub trait BanchoStateBackgroundService {
    fn start_all(&self, configs: BanchoStateBackgroundServiceConfigs);
    fn start_user_sessions_recycle(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    );
    fn stop_user_sessions_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError>;

    fn start_notify_messages_recyce(
        &self,
        config: Arc<LoopBackgroundTaskConfig>,
    );

    fn stop_notify_messages_recyce(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError>;
}

#[async_trait]
pub trait UserSessionsService {
    fn user_sessions(&self) -> &Arc<UserSessions>;

    fn notify_queue(&self) -> &Arc<Mutex<MessageQueue<Packet, i32, Ulid>>>;

    async fn create(&self, create_session: CreateSessionDto) -> Arc<Session>;

    async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>>;

    async fn get(&self, query: &UserQuery) -> Option<Arc<Session>>;

    async fn exists(&self, query: &UserQuery) -> bool;

    async fn clear(&self);

    fn len(&self) -> usize;
}

#[async_trait]
pub trait BanchoStateService {
    async fn broadcast_bancho_packets(
        &self,
        request: BroadcastBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn enqueue_bancho_packets(
        &self,
        request: EnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn batch_enqueue_bancho_packets(
        &self,
        request: BatchEnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn dequeue_bancho_packets(
        &self,
        request: DequeueBanchoPacketsRequest,
    ) -> Result<BanchoPackets, BanchoStateError>;

    async fn create_user_session(
        &self,
        request: CreateUserSessionRequest,
    ) -> Result<CreateUserSessionResponse, BanchoStateError>;

    async fn delete_user_session(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn check_user_token(
        &self,
        token: BanchoClientToken,
    ) -> Result<CheckUserTokenResponse, BanchoStateError>;

    async fn is_user_online(
        &self,
        query: UserQuery,
    ) -> Result<UserOnlineResponse, BanchoStateError>;

    async fn get_user_session(
        &self,
        query: UserQuery,
    ) -> Result<GetUserSessionResponse, BanchoStateError>;

    async fn get_user_session_with_fields(
        &self,
        raw_query: RawUserQueryWithFields,
    ) -> Result<GetUserSessionResponse, BanchoStateError>;

    async fn channel_update_notify(
        &self,
        request: ChannelUpdateNotifyRequest,
    ) -> Result<ChannelUpdateNotifyResponse, BanchoStateError>;

    async fn get_all_sessions(
        &self,
    ) -> Result<GetAllSessionsResponse, BanchoStateError>;

    async fn send_user_stats_packet(
        &self,
        request: SendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn batch_send_user_stats_packet(
        &self,
        request: BatchSendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn send_all_presences(
        &self,
        request: SendAllPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn batch_send_presences(
        &self,
        request: BatchSendPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn update_presence_filter(
        &self,
        request: UpdatePresenceFilterRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;

    async fn update_user_bancho_status(
        &self,
        request: UpdateUserBanchoStatusRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}
