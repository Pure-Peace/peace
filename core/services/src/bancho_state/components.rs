use bitmask_enum::bitmask;
use chrono::{DateTime, Utc};
use peace_pb::bancho_state_rpc::{
    ConnectionInfo, CreateUserSessionRequest, UserQuery,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use tonic::Status;
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
pub enum UserPresenceFilter {
    #[default]
    None    = 0,
    All     = 1,
    Friends = 2,
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

#[derive(Debug, Default, Clone)]
pub struct UserPlayingStats {
    pub rank: i32,
    pub pp_v2: f32,
    pub accuracy: f32,
    pub total_hits: i32,
    pub total_score: i64,
    pub ranked_score: i64,
    pub playcount: i32,
    pub playtime: i64,
    pub max_combo: i32,
}

#[derive(Debug, Default, Clone)]
pub struct BanchoStatus {
    pub online_status: UserOnlineStatus,
    pub description: String,
    pub beatmap_id: i32,
    pub beatmap_md5: String,
    pub mods: Mods,
    pub mode: GameMode,
}

/// User object representing a connected client.
#[derive(Debug, Default, Clone)]
pub struct User {
    /// Unique user ID.
    pub id: i32,
    /// User's username.
    pub username: String,
    /// User's username in unicode, if available.
    pub username_unicode: Option<String>,
    /// User's privileges level.
    pub privileges: i32,
    /// The timestamp of when the user was last active.
    pub last_active: DateTime<Utc>,

    pub bancho_status: BanchoStatus,

    pub playing_stats: UserPlayingStats,
}

impl User {
    /// Update the last active timestamp to the current time.
    #[inline]
    pub fn update_active(&mut self) {
        self.last_active = Utc::now();
    }
}

pub type PacketData = Vec<u8>;
pub type PacketDataPtr = Arc<Vec<u8>>;
pub type PacketsQueue = Vec<PacketDataPtr>;

#[derive(Debug, Default, Clone)]
pub struct Session {
    /// Unique session ID of session.
    pub id: String,
    /// Unique user ID.
    pub user_id: i32,
    /// Information about the user's connection.
    pub connection_info: ConnectionInfo,
    pub user: Arc<RwLock<User>>,
    pub packets_queue: Arc<Mutex<PacketsQueue>>,
    /// The timestamp of when the session was created.
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user: User, connection_info: ConnectionInfo) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user.id,
            connection_info,
            user: Arc::new(RwLock::new(user)),
            packets_queue: Arc::new(Mutex::new(PacketsQueue::new())),
            created_at: Utc::now(),
        }
    }

    pub fn from_request(
        request: CreateUserSessionRequest,
    ) -> Result<Self, Status> {
        let CreateUserSessionRequest {
            user_id,
            username,
            username_unicode,
            privileges,
            connection_info,
        } = request;

        Ok(Self::new(
            User {
                id: user_id,
                username,
                username_unicode,
                privileges,
                last_active: Utc::now(),
                ..Default::default()
            },
            connection_info
                .ok_or(Status::invalid_argument("invalid connection info"))?,
        ))
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
    pub async fn create(&mut self, session: Session) -> String {
        // Delete any existing session with the same user ID
        self.delete(&UserQuery::UserId(session.user_id)).await;

        let session_id = session.id.clone();

        // Clone the relevant data from the user struct
        let (username, username_unicode) = {
            let user = session.user.read().await;
            (user.username.clone(), user.username_unicode.clone())
        };

        // Create a new pointer to the user data
        let session = Arc::new(session);

        // Insert the user data into the relevant hash maps
        self.indexed_by_session_id.insert(session_id.clone(), session.clone());
        self.indexed_by_user_id.insert(session.user_id, session.clone());
        self.indexed_by_username.insert(username, session.clone());
        username_unicode
            .and_then(|s| self.indexed_by_username_unicode.insert(s, session));

        // Increment the length of the collection
        self.len += 1;

        // Return the session ID of the created or updated session
        session_id
    }

    #[inline]
    pub async fn delete(&mut self, query: &UserQuery) -> Option<Arc<Session>> {
        let Session { id, user, .. } = &*self.get(query)?;
        let user = user.read().await;

        let mut removed = None;

        self.indexed_by_user_id
            .remove(&user.id)
            .and_then(|s| Some(removed = Some(s)));

        self.indexed_by_username
            .remove(&user.username)
            .and_then(|s| Some(removed = Some(s)));

        self.indexed_by_session_id
            .remove(id)
            .and_then(|s| Some(removed = Some(s)));

        user.username_unicode
            .as_ref()
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
