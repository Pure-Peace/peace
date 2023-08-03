use super::{BanchoStateBackgroundServiceConfigs, BanchoStateServiceSnapshot};
use crate::{BanchoExtend, BanchoSession, BanchoStateError, UserSessions};
use async_trait::async_trait;
use infra_packets::Packet;
use infra_services::ServiceSnapshot;
use infra_users::CreateSessionDto;
use peace_domain::bancho::BanchoClientToken;
use peace_message_queue::{MessageData, MessageQueue};
use peace_pb::{bancho_state::*, base::ExecSuccess};
use peace_snapshot::{CreateSnapshot, SaveSnapshotTo};
use peace_unique_id::Ulid;
use std::sync::Arc;
use tools::async_collections::{
    BackgroundTask, BackgroundTaskError, CommonRecycleBackgroundTaskConfig,
    LoopBackgroundTaskConfig,
};

pub type BanchoMessageQueue = MessageQueue<Packet, i32, Ulid>;
pub type BanchoMessageData = MessageData<Packet, i32, Ulid>;

pub type DynBanchoStateService = Arc<dyn BanchoStateService + Send + Sync>;

pub type DynBanchoStateBackgroundService =
    Arc<dyn BanchoStateBackgroundService + Send + Sync>;

pub type DynUserSessionsService = Arc<dyn UserSessionsService + Send + Sync>;

#[async_trait]
pub trait BanchoStateBackgroundService:
    UserSessionsCleaner + NotifyMessagesCleaner
{
    fn start_all(&self, configs: BanchoStateBackgroundServiceConfigs);
}

#[async_trait]
pub trait UserSessionsCleaner {
    fn start_user_sessions_recycle(
        &self,
        config: Arc<CommonRecycleBackgroundTaskConfig>,
    );
    fn stop_user_sessions_recycle(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError>;
}

#[async_trait]
pub trait NotifyMessagesCleaner {
    fn start_notify_messages_recyce(
        &self,
        config: Arc<LoopBackgroundTaskConfig>,
    );

    fn stop_notify_messages_recyce(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError>;
}

pub trait UserSessionsStore {
    fn user_sessions(&self) -> &Arc<UserSessions>;
}

pub trait NotifyMessagesQueue {
    fn notify_queue(&self) -> &Arc<BanchoMessageQueue>;
}

#[async_trait]
pub trait UserSessionsService:
    UserSessionsCreate
    + UserSessionsDelete
    + UserSessionsGet
    + UserSessionsExists
    + UserSessionsClear
    + UserSessionsCount
{
}

pub trait UserSessionsCount: UserSessionsStore {
    #[inline]
    fn length(&self) -> usize {
        self.user_sessions().length()
    }
}

#[async_trait]
pub trait UserSessionsClear: UserSessionsStore {
    #[inline]
    async fn clear(&self) {
        self.user_sessions().clear().await
    }
}

#[async_trait]
pub trait UserSessionsGet: UserSessionsStore {
    #[inline]
    async fn get(&self, query: &UserQuery) -> Option<Arc<BanchoSession>> {
        self.user_sessions().get(query).await
    }
}

#[async_trait]
pub trait UserSessionsDelete: UserSessionsStore + NotifyMessagesQueue {
    #[inline]
    async fn delete(&self, query: &UserQuery) -> Option<Arc<BanchoSession>> {
        const LOG_TARGET: &str = "bancho_state::user_sessions::delete_session";

        let session = self.user_sessions().delete(query).await?;

        self.notify_queue().write().await.push_message(
            bancho_packets::server::UserLogout::pack(session.user_id).into(),
            None,
        );

        info!(
            target: LOG_TARGET,
            "Session deleted: {} [{}] ({})",
            session.username.load(),
            session.user_id,
            session.id
        );

        Some(session)
    }
}

#[async_trait]
pub trait UserSessionsCreate: UserSessionsStore + NotifyMessagesQueue {
    #[inline]
    async fn create(
        &self,
        create_session: CreateSessionDto<BanchoExtend>,
    ) -> Arc<BanchoSession> {
        const LOG_TARGET: &str = "bancho_state::user_sessions::create_session";
        const PRESENCE_SHARD_SIZE: usize = 512;

        let session = self
            .user_sessions()
            .create(BanchoSession::new(create_session).into())
            .await;

        let weak = Arc::downgrade(&session);

        self.notify_queue().write().await.push_message_excludes(
            bancho_packets::server::UserPresenceSingle::pack(session.user_id)
                .into(),
            [session.user_id],
            Some(Arc::new(move |_| weak.upgrade().is_some())),
        );

        let online_users = {
            self.user_sessions()
                .read()
                .await
                .keys()
                .copied()
                .collect::<Vec<i32>>()
        };
        let online_users_len = online_users.len();

        let mut presence_shard_count = online_users_len / PRESENCE_SHARD_SIZE;
        if (online_users_len % PRESENCE_SHARD_SIZE) > 0 {
            presence_shard_count += 1
        };

        let session_info = session.user_info_packets();

        let pre_alloc_size = session_info.len()
            + (9 + presence_shard_count * PRESENCE_SHARD_SIZE * 4);

        let mut pending_packets = Vec::with_capacity(pre_alloc_size);

        pending_packets.push(session_info.into());

        for shard in online_users.chunks(PRESENCE_SHARD_SIZE) {
            pending_packets.push(
                bancho_packets::server::UserPresenceBundle::pack(shard).into(),
            )
        }

        session.extends.packets_queue.enqueue_packets(pending_packets).await;

        info!(
            target: LOG_TARGET,
            "Session created: {} [{}] ({})",
            session.username.load(),
            session.user_id,
            session.id
        );

        session
    }
}

#[async_trait]
pub trait UserSessionsExists: UserSessionsStore {
    #[inline]
    async fn exists(&self, query: &UserQuery) -> bool {
        self.user_sessions().exists(query).await
    }
}

#[async_trait]
pub trait BanchoStateService:
    UpdateUserBanchoStatus
    + UpdatePresenceFilter
    + BatchSendPresences
    + SendAllPresences
    + BatchSendUserStatsPacket
    + SendUserStatsPacket
    + GetAllSessions
    + GetUserSessionWithFields
    + GetUserSession
    + IsUserOnline
    + CheckUserToken
    + DeleteUserSession
    + CreateUserSession
    + DequeueBanchoPackets
    + BatchEnqueueBanchoPackets
    + EnqueueBanchoPackets
    + BroadcastBanchoPackets
    + CreateSnapshot<BanchoStateServiceSnapshot>
    + SaveSnapshotTo<BanchoStateServiceSnapshot>
    + ServiceSnapshot
{
}

#[async_trait]
pub trait UpdateUserBanchoStatus {
    async fn update_user_bancho_status(
        &self,
        request: UpdateUserBanchoStatusRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait UpdatePresenceFilter {
    async fn update_presence_filter(
        &self,
        request: UpdatePresenceFilterRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait BatchSendPresences {
    async fn batch_send_presences(
        &self,
        request: BatchSendPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait SendAllPresences {
    async fn send_all_presences(
        &self,
        request: SendAllPresencesRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait BatchSendUserStatsPacket {
    async fn batch_send_user_stats_packet(
        &self,
        request: BatchSendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait SendUserStatsPacket {
    async fn send_user_stats_packet(
        &self,
        request: SendUserStatsPacketRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait GetAllSessions {
    async fn get_all_sessions(
        &self,
    ) -> Result<GetAllSessionsResponse, BanchoStateError>;
}

#[async_trait]
pub trait GetUserSessionWithFields {
    async fn get_user_session_with_fields(
        &self,
        raw_query: RawUserQueryWithFields,
    ) -> Result<GetUserSessionResponse, BanchoStateError>;
}

#[async_trait]
pub trait GetUserSession {
    async fn get_user_session(
        &self,
        query: UserQuery,
    ) -> Result<GetUserSessionResponse, BanchoStateError>;
}

#[async_trait]
pub trait IsUserOnline {
    async fn is_user_online(
        &self,
        query: UserQuery,
    ) -> Result<UserOnlineResponse, BanchoStateError>;
}

#[async_trait]
pub trait CheckUserToken {
    async fn check_user_token(
        &self,
        token: BanchoClientToken,
    ) -> Result<CheckUserTokenResponse, BanchoStateError>;
}

#[async_trait]
pub trait DeleteUserSession {
    async fn delete_user_session(
        &self,
        query: UserQuery,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait CreateUserSession {
    async fn create_user_session(
        &self,
        request: CreateUserSessionRequest,
    ) -> Result<CreateUserSessionResponse, BanchoStateError>;
}

#[async_trait]
pub trait DequeueBanchoPackets {
    async fn dequeue_bancho_packets(
        &self,
        request: DequeueBanchoPacketsRequest,
    ) -> Result<BanchoPackets, BanchoStateError>;
}

#[async_trait]
pub trait BatchEnqueueBanchoPackets {
    async fn batch_enqueue_bancho_packets(
        &self,
        request: BatchEnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait EnqueueBanchoPackets {
    async fn enqueue_bancho_packets(
        &self,
        request: EnqueueBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}

#[async_trait]
pub trait BroadcastBanchoPackets {
    async fn broadcast_bancho_packets(
        &self,
        request: BroadcastBanchoPacketsRequest,
    ) -> Result<ExecSuccess, BanchoStateError>;
}
