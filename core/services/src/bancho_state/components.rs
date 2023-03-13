use super::CreateSessionError;
use bitmask_enum::bitmask;
use chrono::{DateTime, Utc};
use peace_domain::bancho_state::ConnectionInfo;
use peace_pb::bancho_state::{CreateUserSessionRequest, UserQuery};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};
use tools::{
    atomic::{Atomic, AtomicOption, AtomicValue, Bool, F32, I32, I64},
    Timestamp,
};
use uuid::Uuid;

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Hash, Serialize, Deserialize)]
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
#[derive(Default, Deserialize)]
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

impl serde::Serialize for Mods {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Serialize, Deserialize)]
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
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Serialize, Deserialize)]
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

#[derive(Debug, Default, Serialize)]
pub struct ModeStats {
    pub rank: I32,
    pub pp_v2: F32,
    pub accuracy: F32,
    pub total_hits: I32,
    pub total_score: I64,
    pub ranked_score: I64,
    pub playcount: I32,
    pub playtime: I64,
    pub max_combo: I32,
}

#[derive(Debug, Default, Serialize)]
pub struct BanchoStatus {
    pub online_status: Atomic<UserOnlineStatus>,
    pub description: Atomic<String>,
    pub beatmap_id: I32,
    pub beatmap_md5: Atomic<String>,
    pub mods: Atomic<Mods>,
    pub mode: Atomic<GameMode>,
}

impl BanchoStatus {
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
        self.online_status.set(online_status.into());
        self.description.set(description.into());
        self.beatmap_id.set(beatmap_id.into());
        self.beatmap_md5.set(beatmap_md5.into());
        self.mods.set(mods.into());
        self.mode.set(mode.into());
    }
}

pub type PacketData = Vec<u8>;
pub type PacketDataPtr = Arc<Vec<u8>>;
pub type PacketsQueue = Vec<PacketDataPtr>;

#[derive(Debug, Default, Serialize)]
pub struct UserModeStatSets {
    pub standard: Option<ModeStats>,
    pub taiko: Option<ModeStats>,
    pub fruits: Option<ModeStats>,
    pub mania: Option<ModeStats>,
    pub standard_relax: Option<ModeStats>,
    pub taiko_relax: Option<ModeStats>,
    pub fruits_relax: Option<ModeStats>,
    pub standard_autopilot: Option<ModeStats>,
    pub standard_score_v2: Option<ModeStats>,
}

#[derive(Debug, Default, Serialize)]
pub struct Session {
    /// Unique session ID of session.
    pub id: String,
    /// Unique user ID.
    pub user_id: i32,
    /// User's username.
    pub username: Atomic<String>,
    /// User's username in unicode, if available.
    pub username_unicode: AtomicOption<String>,
    /// User's privileges level.
    pub privileges: I32,
    pub client_version: String,
    pub utc_offset: u8,
    pub presence_filter: Atomic<PresenceFilter>,
    pub display_city: bool,
    pub only_friend_pm_allowed: Bool,
    pub bancho_status: BanchoStatus,
    pub mode_stat_sets: UserModeStatSets,
    /// Information about the user's connection.
    pub connection_info: ConnectionInfo,
    #[serde(skip_serializing)]
    pub packets_queue: Mutex<PacketsQueue>,
    /// The timestamp of when the session was created.
    pub created_at: DateTime<Utc>,
    pub last_active: I64,
}

impl Session {
    #[inline]
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
            username: username.into(),
            username_unicode: username_unicode.into(),
            privileges: privileges.into(),
            client_version,
            utc_offset,
            presence_filter: Default::default(),
            display_city,
            only_friend_pm_allowed: only_friend_pm_allowed.into(),
            bancho_status: BanchoStatus::default().into(),
            mode_stat_sets: UserModeStatSets::default().into(),
            connection_info,
            packets_queue: Mutex::new(PacketsQueue::new()),
            created_at: Utc::now(),
            last_active: Timestamp::now().into(),
        }
    }

    #[inline]
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
    pub fn update_active(&self) {
        self.last_active.set(Timestamp::now());
    }

    #[inline]
    pub async fn queued_packets(&self) -> usize {
        self.packets_queue.lock().await.len()
    }

    #[inline]
    pub async fn push_packet(&self, packet: PacketDataPtr) -> usize {
        let mut queue = self.packets_queue.lock().await;
        queue.push(packet);
        queue.len()
    }

    #[inline]
    pub async fn enqueue_packets<I>(&self, packets: I) -> usize
    where
        I: IntoIterator<Item = PacketDataPtr>,
    {
        let mut queue = self.packets_queue.lock().await;
        queue.extend(packets);
        queue.len()
    }

    #[inline]
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
    pub fn mode_stats(&self) -> Option<&ModeStats> {
        match &self.bancho_status.mode.load().as_ref() {
            GameMode::Standard => self.mode_stat_sets.standard.as_ref(),
            GameMode::Taiko => self.mode_stat_sets.taiko.as_ref(),
            GameMode::Fruits => self.mode_stat_sets.fruits.as_ref(),
            GameMode::Mania => self.mode_stat_sets.mania.as_ref(),
            GameMode::StandardRelax => {
                self.mode_stat_sets.standard_relax.as_ref()
            },
            GameMode::TaikoRelax => self.mode_stat_sets.taiko_relax.as_ref(),
            GameMode::FruitsRelax => self.mode_stat_sets.fruits_relax.as_ref(),
            GameMode::StandardAutopilot => {
                self.mode_stat_sets.standard_autopilot.as_ref()
            },
            GameMode::StandardScoreV2 => {
                self.mode_stat_sets.standard_score_v2.as_ref()
            },
        }
    }

    #[inline]
    pub fn user_stats_packet(&self) -> Vec<u8> {
        let bancho_status = &self.bancho_status;
        let mode_stats = self.mode_stats();

        bancho_packets::server::user_stats(
            self.user_id,
            bancho_status.online_status.load().val(),
            bancho_status.description.to_string(),
            bancho_status.beatmap_md5.to_string(),
            bancho_status.mods.load().bits(),
            bancho_status.mode.load().val(),
            bancho_status.beatmap_id.val(),
            mode_stats.map(|s| s.ranked_score.val()).unwrap_or_default(),
            mode_stats.map(|s| s.accuracy.val()).unwrap_or_default(),
            mode_stats.map(|s| s.playcount.val()).unwrap_or_default(),
            mode_stats.map(|s| s.total_score.val()).unwrap_or_default(),
            mode_stats.map(|s| s.rank.val()).unwrap_or_default(),
            mode_stats.map(|s| s.pp_v2.val() as i16).unwrap_or_default(),
        )
    }

    #[inline]
    pub fn user_presence_packet(&self) -> Vec<u8> {
        bancho_packets::server::user_presence(
            self.user_id,
            self.username.to_string(),
            self.utc_offset,
            0, // todo
            1, // todo
            self.connection_info.location.longitude as f32,
            self.connection_info.location.latitude as f32,
            self.mode_stats().map(|s| s.rank.val()).unwrap_or_default(),
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
        self.indexed_by_username
            .insert(session.username.to_string(), session.clone());
        session.username_unicode.load().as_ref().and_then(|s| {
            self.indexed_by_username_unicode
                .insert(s.to_string(), session.clone())
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
            UserQuery::Username(username) => {
                self.indexed_by_username.get(username)
            },
            UserQuery::UsernameUnicode(username_unicode) => {
                self.indexed_by_username_unicode.get(username_unicode)
            },
            UserQuery::SessionId(session_id) => {
                self.indexed_by_session_id.get(session_id)
            },
        }
        .cloned()
    }

    #[inline]
    pub fn exists(&self, query: &UserQuery) -> bool {
        match query {
            UserQuery::UserId(user_id) => {
                self.indexed_by_user_id.contains_key(user_id)
            },
            UserQuery::Username(username) => {
                self.indexed_by_username.contains_key(username)
            },
            UserQuery::UsernameUnicode(username_unicode) => {
                self.indexed_by_username_unicode.contains_key(username_unicode)
            },
            UserQuery::SessionId(session_id) => {
                self.indexed_by_session_id.contains_key(session_id)
            },
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
