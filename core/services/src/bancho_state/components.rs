use super::CreateSessionError;
use arc_swap::{ArcSwap, ArcSwapOption};
use atomic_float::AtomicF32;
use bitmask_enum::bitmask;
use chrono::{DateTime, Utc};
use peace_domain::bancho_state::ConnectionInfo;
use peace_pb::bancho_state::{CreateUserSessionRequest, UserQuery};
use std::{
    collections::HashMap,
    sync::{
        atomic::{self, AtomicBool, AtomicI32, AtomicI64},
        Arc,
    },
};
use tokio::sync::{Mutex, MutexGuard};
use tools::Timestamp;
use uuid::Uuid;

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Hash)]
pub enum GameMode {
    #[default]
    Standard            = 0,
    Taiko               = 1,
    Fruits              = 2,
    Mania               = 3,

    StandardRelax       = 4,
    TaikoRelax          = 5,
    FruitsRelax         = 6,
    StandardAutopilot   = 8,

    StandardScoreV2     = 12,
}

impl GameMode {
    #[inline]
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

#[rustfmt::skip]
#[derive(Default)]
#[bitmask(u32)]
pub enum Mods {
    #[default]
    NoMod         = 0,
    NoFail        = 1 << 0,
    Easy          = 1 << 1,
    TouchScreen   = 1 << 2,
    Hidden        = 1 << 3,
    HardRock      = 1 << 4,
    SuddenDeath   = 1 << 5,
    DoubleTime    = 1 << 6,
    Relax         = 1 << 7,
    HalfTime      = 1 << 8,
    NightCore     = 1 << 9,
    FlashLight    = 1 << 10,
    Auto          = 1 << 11,
    SpunOut       = 1 << 12,
    AutoPilot     = 1 << 13,
    Perfect       = 1 << 14,
    Key4          = 1 << 15,
    Key5          = 1 << 16,
    Key6          = 1 << 17,
    Key7          = 1 << 18,
    Key8          = 1 << 19,
    FadeIn        = 1 << 20,
    Random        = 1 << 21,
    Cinema        = 1 << 22,
    Target        = 1 << 23,
    Key9          = 1 << 24,
    KeyCoop       = 1 << 25,
    Key1          = 1 << 26,
    Key3          = 1 << 27,
    Key2          = 1 << 28,
    ScoreV2       = 1 << 29,
    Mirror        = 1 << 30,

    KeyMods = Self::Key1.bits
        | Self::Key2.bits
        | Self::Key3.bits
        | Self::Key4.bits
        | Self::Key5.bits
        | Self::Key6.bits
        | Self::Key7.bits
        | Self::Key8.bits
        | Self::Key9.bits,

    ScoreIncrease = Self::Hidden.bits
        | Self::HardRock.bits
        | Self::FadeIn.bits
        | Self::DoubleTime.bits
        | Self::FlashLight.bits,

    SpeedChanging =
        Self::DoubleTime.bits | Self::NightCore.bits | Self::HalfTime.bits,

    StandardOnly = Self::AutoPilot.bits | Self::SpunOut.bits | Self::Target.bits,
    ManiaOnly = Self::Mirror.bits
        | Self::Random.bits
        | Self::FadeIn.bits
        | Self::KeyMods.bits,
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive)]
pub enum UserOnlineStatus {
    #[default]
    Idle          = 0,
    Afk           = 1,
    Playing       = 2,
    Editing       = 3,
    Modding       = 4,
    Multiplayer   = 5,
    Watching      = 6,
    Unknown       = 7,
    Testing       = 8,
    Submitting    = 9,
    Paused        = 10,
    Lobby         = 11,
    Multiplaying  = 12,
    Direct        = 13,
}

impl UserOnlineStatus {
    #[inline]
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive)]
pub enum PresenceFilter {
    #[default]
    None    = 0,
    All     = 1,
    Friends = 2,
}

impl PresenceFilter {
    #[inline]
    pub fn val(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Default)]
pub struct ModeStats {
    pub rank: AtomicI32,
    pub pp_v2: AtomicF32,
    pub accuracy: AtomicF32,
    pub total_hits: AtomicI32,
    pub total_score: AtomicI64,
    pub ranked_score: AtomicI64,
    pub playcount: AtomicI32,
    pub playtime: AtomicI64,
    pub max_combo: AtomicI32,
}

impl ModeStats {
    #[inline]
    pub fn rank(&self) -> i32 {
        self.rank.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn pp_v2(&self) -> f32 {
        self.pp_v2.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn accuracy(&self) -> f32 {
        self.accuracy.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn total_hits(&self) -> i32 {
        self.total_hits.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn total_score(&self) -> i64 {
        self.total_score.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn ranked_score(&self) -> i64 {
        self.ranked_score.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn playcount(&self) -> i32 {
        self.playcount.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn playtime(&self) -> i64 {
        self.playtime.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn max_combo(&self) -> i32 {
        self.max_combo.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_rank(&self, rank: i32) {
        self.rank.store(rank, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_pp_v2(&self, pp_v2: f32) {
        self.pp_v2.store(pp_v2, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_accuracy(&self, accuracy: f32) {
        self.accuracy.store(accuracy, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_total_hits(&self, total_hits: i32) {
        self.total_hits.store(total_hits, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_total_score(&self, total_score: i64) {
        self.total_score.store(total_score, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_ranked_score(&self, ranked_score: i64) {
        self.ranked_score.store(ranked_score, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_playcount(&self, playcount: i32) {
        self.playcount.store(playcount, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_playtime(&self, playtime: i64) {
        self.playtime.store(playtime, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_max_combo(&self, max_combo: i32) {
        self.max_combo.store(max_combo, atomic::Ordering::SeqCst)
    }
}

#[derive(Debug, Default)]
pub struct BanchoStatus {
    pub online_status: ArcSwap<UserOnlineStatus>,
    pub description: ArcSwap<String>,
    pub beatmap_id: AtomicI32,
    pub beatmap_md5: ArcSwap<String>,
    pub mods: ArcSwap<Mods>,
    pub mode: ArcSwap<GameMode>,
}

impl BanchoStatus {
    #[inline]
    pub fn online_status(&self) -> UserOnlineStatus {
        self.online_status.load().as_ref().clone()
    }

    #[inline]
    pub fn description(&self) -> String {
        self.description.load().as_ref().clone()
    }

    #[inline]
    pub fn beatmap_id(&self) -> i32 {
        self.beatmap_id.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn beatmap_md5(&self) -> String {
        self.beatmap_md5.load().as_ref().clone()
    }

    #[inline]
    pub fn mods(&self) -> Mods {
        self.mods.load().as_ref().clone()
    }

    #[inline]
    pub fn mode(&self) -> GameMode {
        self.mode.load().as_ref().clone()
    }

    #[inline]
    pub fn update_all(
        &self,
        online_status: UserOnlineStatus,
        description: String,
        beatmap_id: i32,
        beatmap_md5: String,
        mods: Mods,
        mode: GameMode,
    ) {
        self.set_online_status(online_status);
        self.set_description(description);
        self.set_beatmap_id(beatmap_id);
        self.set_beatmap_md5(beatmap_md5);
        self.set_mods(mods);
        self.set_mode(mode);
    }

    #[inline]
    pub fn set_online_status(&self, online_status: UserOnlineStatus) {
        self.online_status.store(online_status.into())
    }

    #[inline]
    pub fn set_description(&self, description: String) {
        self.description.store(description.into())
    }

    #[inline]
    pub fn set_beatmap_id(&self, beatmap_id: i32) {
        self.beatmap_id.store(beatmap_id, atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn set_beatmap_md5(&self, beatmap_md5: String) {
        self.beatmap_md5.store(beatmap_md5.into())
    }

    #[inline]
    pub fn set_mods(&self, mods: Mods) {
        self.mods.store(mods.into())
    }

    #[inline]
    pub fn set_mode(&self, mode: GameMode) {
        self.mode.store(mode.into())
    }
}

pub type PacketData = Vec<u8>;
pub type PacketDataPtr = Arc<Vec<u8>>;
pub type PacketsQueue = Vec<PacketDataPtr>;

#[derive(Debug, Default)]
pub struct UserModeStatSets {
    pub standard: ModeStats,
    pub taiko: ModeStats,
    pub fruits: ModeStats,
    pub mania: ModeStats,
    pub standard_relax: ModeStats,
    pub taiko_relax: ModeStats,
    pub fruits_relax: ModeStats,
    pub standard_autopilot: ModeStats,
    pub standard_score_v2: ModeStats,
}

#[derive(Debug, Default)]
pub struct Session {
    /// Unique session ID of session.
    pub id: String,
    /// Unique user ID.
    pub user_id: i32,
    /// User's username.
    pub username: ArcSwap<String>,
    /// User's username in unicode, if available.
    pub username_unicode: ArcSwapOption<String>,
    /// User's privileges level.
    pub privileges: AtomicI32,
    pub client_version: String,
    pub utc_offset: u8,
    pub presence_filter: ArcSwap<PresenceFilter>,
    pub display_city: bool,
    pub only_friend_pm_allowed: AtomicBool,
    pub bancho_status: BanchoStatus,
    pub mode_stat_sets: UserModeStatSets,
    /// Information about the user's connection.
    pub connection_info: ConnectionInfo,
    pub packets_queue: Mutex<PacketsQueue>,
    /// The timestamp of when the session was created.
    pub created_at: DateTime<Utc>,
    pub last_active: AtomicI64,
}

impl Session {
    pub fn new(
        user_id: i32,
        username: String,
        username_unicode: Option<String>,
        privileges: i32,
        client_version: String,
        utc_offset: u8,
        display_city: bool,
        only_friend_pm_allowed: bool,
        connection_info: ConnectionInfo,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            username: ArcSwap::new(Arc::new(username)),
            username_unicode: username_unicode
                .map(|s| ArcSwapOption::new(Some(Arc::new(s))))
                .unwrap_or_default(),
            privileges: AtomicI32::new(privileges),
            client_version,
            utc_offset,
            presence_filter: Default::default(),
            display_city,
            only_friend_pm_allowed: AtomicBool::new(only_friend_pm_allowed),
            bancho_status: BanchoStatus::default().into(),
            mode_stat_sets: UserModeStatSets::default().into(),
            connection_info,
            packets_queue: Mutex::new(PacketsQueue::new()),
            created_at: Utc::now(),
            last_active: AtomicI64::new(Timestamp::now()),
        }
    }

    pub fn from_request(
        request: CreateUserSessionRequest,
    ) -> Result<Self, CreateSessionError> {
        let CreateUserSessionRequest {
            user_id,
            username,
            username_unicode,
            privileges,
            client_version,
            utc_offset,
            display_city,
            only_friend_pm_allowed,
            connection_info,
        } = request;

        Ok(Self::new(
            user_id,
            username,
            username_unicode,
            privileges,
            client_version,
            utc_offset as u8,
            display_city,
            only_friend_pm_allowed,
            connection_info
                .ok_or(CreateSessionError::InvalidConnectionInfo)?
                .into(),
        ))
    }

    #[inline]
    pub fn username(&self) -> String {
        self.username.load().to_string()
    }

    #[inline]
    pub fn username_unicode(&self) -> Option<String> {
        self.username_unicode.load().as_deref().map(|s| s.to_string())
    }

    #[inline]
    pub fn privileges(&self) -> i32 {
        self.privileges.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn presence_filter(&self) -> PresenceFilter {
        *self.presence_filter.load().as_ref()
    }

    #[inline]
    pub fn set_presence_filter(&self, presence_filter: PresenceFilter) {
        self.presence_filter.store(Arc::new(presence_filter))
    }

    #[inline]
    pub fn only_friend_pm_allowed(&self) -> bool {
        self.only_friend_pm_allowed.load(atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn update_active(&self) {
        self.last_active.store(Timestamp::now(), atomic::Ordering::SeqCst);
    }

    #[inline]
    pub fn last_active(&self) -> i64 {
        self.last_active.load(atomic::Ordering::SeqCst)
    }

    pub async fn queued_packets(&self) -> usize {
        self.packets_queue.lock().await.len()
    }

    pub async fn push_packet(&self, packet: PacketDataPtr) -> usize {
        let mut queue = self.packets_queue.lock().await;
        queue.push(packet);
        queue.len()
    }

    pub async fn enqueue_packets<I>(&self, packets: I) -> usize
    where
        I: IntoIterator<Item = PacketDataPtr>,
    {
        let mut queue = self.packets_queue.lock().await;
        queue.extend(packets);
        queue.len()
    }

    pub async fn dequeue_packet(
        &self,
        queue_lock: Option<&mut MutexGuard<'_, PacketsQueue>>,
    ) -> Option<PacketDataPtr> {
        fn dequeue(
            queue: &mut MutexGuard<'_, PacketsQueue>,
        ) -> Option<PacketDataPtr> {
            if !queue.is_empty() {
                Some(queue.remove(0))
            } else {
                None
            }
        }

        match queue_lock {
            Some(mut queue) => dequeue(&mut queue),
            None => dequeue(&mut self.packets_queue.lock().await),
        }
    }

    #[inline]
    pub fn mode_stats(&self) -> &ModeStats {
        match &self.bancho_status.mode() {
            GameMode::Standard => &self.mode_stat_sets.standard,
            GameMode::Taiko => &self.mode_stat_sets.taiko,
            GameMode::Fruits => &self.mode_stat_sets.fruits,
            GameMode::Mania => &self.mode_stat_sets.mania,
            GameMode::StandardRelax => &self.mode_stat_sets.standard_relax,
            GameMode::TaikoRelax => &self.mode_stat_sets.taiko_relax,
            GameMode::FruitsRelax => &self.mode_stat_sets.fruits_relax,
            GameMode::StandardAutopilot =>
                &self.mode_stat_sets.standard_autopilot,
            GameMode::StandardScoreV2 => &self.mode_stat_sets.standard_score_v2,
        }
    }

    pub fn user_stats_packet(&self) -> Vec<u8> {
        let bancho_status = &self.bancho_status;
        let mode_stats = self.mode_stats();

        bancho_packets::server::user_stats(
            self.user_id,
            bancho_status.online_status.load().val(),
            bancho_status.description(),
            bancho_status.beatmap_md5(),
            bancho_status.mods.load().bits(),
            bancho_status.mode.load().val(),
            bancho_status.beatmap_id(),
            mode_stats.ranked_score(),
            mode_stats.accuracy(),
            mode_stats.playcount(),
            mode_stats.total_score(),
            mode_stats.rank(),
            mode_stats.pp_v2() as i16,
        )
    }
}

/// A struct representing a collection of user sessions.
#[derive(Debug, Default, Clone)]
pub struct UserSessionsInner {
    /// A hash map that maps session IDs to user data
    pub indexed_by_session_id: HashMap<String, Arc<Session>>,
    /// A hash map that maps user IDs to user data
    pub indexed_by_user_id: HashMap<i32, Arc<Session>>,
    /// A hash map that maps usernames to user data
    pub indexed_by_username: HashMap<String, Arc<Session>>,
    /// A hash map that maps Unicode usernames to user data
    pub indexed_by_username_unicode: HashMap<String, Arc<Session>>,
    /// The number of user sessions in the collection
    pub len: usize,
}

impl UserSessionsInner {
    #[inline]
    pub async fn create(&mut self, session: Session) -> Arc<Session> {
        // Delete any existing session with the same user ID
        self.delete(&UserQuery::UserId(session.user_id)).await;

        let session = Arc::new(session);

        // Insert the user data into the relevant hash maps
        self.indexed_by_session_id.insert(session.id.clone(), session.clone());
        self.indexed_by_user_id.insert(session.user_id, session.clone());
        self.indexed_by_username.insert(session.username(), session.clone());
        session.username_unicode().and_then(|s| {
            self.indexed_by_username_unicode.insert(s, session.clone())
        });

        // Increment the length of the collection
        self.len += 1;

        // Return the session ID of the created or updated session
        session
    }

    #[inline]
    pub async fn delete(&mut self, query: &UserQuery) -> Option<Arc<Session>> {
        let session = self.get(query)?;
        self.delete_inner(
            &session.user_id,
            &session.username.load(),
            &session.id,
            session.username_unicode.load().as_deref().map(|s| s.as_str()),
        )
    }

    #[inline]
    pub(crate) fn delete_inner(
        &mut self,
        user_id: &i32,
        username: &str,
        session_id: &str,
        username_unicode: Option<&str>,
    ) -> Option<Arc<Session>> {
        let mut removed = None;

        self.indexed_by_user_id
            .remove(user_id)
            .and_then(|s| Some(removed = Some(s)));

        self.indexed_by_username
            .remove(username)
            .and_then(|s| Some(removed = Some(s)));

        self.indexed_by_session_id
            .remove(session_id)
            .and_then(|s| Some(removed = Some(s)));

        username_unicode
            .and_then(|s| self.indexed_by_username_unicode.remove(s))
            .and_then(|s| Some(removed = Some(s)));

        // Decrease the length of the map if a session was removed.
        if removed.is_some() {
            self.len -= 1;
        }

        removed
    }

    #[inline]
    pub fn get(&self, query: &UserQuery) -> Option<Arc<Session>> {
        match query {
            UserQuery::UserId(user_id) => self.indexed_by_user_id.get(user_id),
            UserQuery::Username(username) =>
                self.indexed_by_username.get(username),
            UserQuery::UsernameUnicode(username_unicode) =>
                self.indexed_by_username_unicode.get(username_unicode),
            UserQuery::SessionId(session_id) =>
                self.indexed_by_session_id.get(session_id),
        }
        .cloned()
    }

    #[inline]
    pub fn exists(&self, query: &UserQuery) -> bool {
        match query {
            UserQuery::UserId(user_id) =>
                self.indexed_by_user_id.contains_key(user_id),
            UserQuery::Username(username) =>
                self.indexed_by_username.contains_key(username),
            UserQuery::UsernameUnicode(username_unicode) =>
                self.indexed_by_username_unicode.contains_key(username_unicode),
            UserQuery::SessionId(session_id) =>
                self.indexed_by_session_id.contains_key(session_id),
        }
    }

    /// Clears all sessions records from the [`UserSessions`].
    #[inline]
    pub fn clear(&mut self) {
        self.indexed_by_session_id.clear();
        self.indexed_by_username.clear();
        self.indexed_by_username_unicode.clear();
        self.indexed_by_session_id.clear();

        self.len = 0;
    }

    /// Returns the number of sessions in the [`UserSessions`].
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}
