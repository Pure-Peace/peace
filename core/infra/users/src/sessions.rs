use crate::UserKey;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use peace_pb::bancho_state::UserQuery;
use peace_snapshot::CreateSnapshot;
use peace_unique_id::Ulid;
use serde::{Deserialize, Serialize};
use tools::{
    atomic::{Atomic, AtomicOption, AtomicValue, I32, U64},
    Timestamp,
};

#[async_trait]
pub trait FromBaseSession {
    async fn from_base_session(s: &BaseSession) -> Self;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateSessionDto<T> {
    pub user_id: i32,
    pub username: String,
    pub username_unicode: Option<String>,
    pub privileges: i32,
    pub extends: T,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BaseSession {
    /// Unique session ID of session.
    pub id: Ulid,
    /// Unique user ID.
    pub user_id: i32,
    /// User's username.
    pub username: Atomic<String>,
    /// User's username in unicode, if available.
    pub username_unicode: AtomicOption<String>,
    /// User's privileges level.
    pub privileges: I32,
    /// The timestamp of when the session was created.
    pub created_at: DateTime<Utc>,
    pub last_active: U64,
}

impl From<BaseSessionData> for BaseSession {
    fn from(u: BaseSessionData) -> Self {
        Self {
            id: u.id,
            user_id: u.user_id,
            username: u.username.into(),
            username_unicode: u.username_unicode.into(),
            privileges: u.privileges.into(),
            created_at: u.created_at,
            last_active: u.last_active.into(),
        }
    }
}

impl UserKey for BaseSession {
    #[inline]
    fn session_id(&self) -> Ulid {
        self.id
    }

    #[inline]
    fn user_id(&self) -> i32 {
        self.user_id
    }

    #[inline]
    fn username(&self) -> String {
        self.username.load().to_string()
    }

    #[inline]
    fn username_unicode(&self) -> Option<String> {
        self.username_unicode.load().as_deref().map(|s| s.to_string())
    }
}

impl BaseSession {
    #[inline]
    pub fn new(
        user_id: i32,
        username: String,
        username_unicode: Option<String>,
        privileges: i32,
    ) -> Self {
        Self {
            id: Ulid::new(),
            user_id,
            username: username.into(),
            username_unicode: username_unicode.into(),
            privileges: privileges.into(),
            created_at: Utc::now(),
            last_active: Timestamp::now().into(),
        }
    }

    #[inline]
    pub fn is_deactive(&self, current_timestamp: u64, deadline: u64) -> bool {
        current_timestamp > self.last_active.val() + deadline
    }

    #[inline]
    pub fn update_active(&self) {
        self.last_active.set(Timestamp::now());
    }

    pub fn to_session_data(&self) -> BaseSessionData {
        BaseSessionData {
            id: self.id,
            user_id: self.user_id,
            username: self.username(),
            username_unicode: self.username_unicode(),
            privileges: self.privileges.val(),
            created_at: self.created_at,
            last_active: self.last_active.val(),
        }
    }
}

#[async_trait]
impl<D> CreateSnapshot<D> for BaseSession
where
    D: FromBaseSession,
{
    async fn create_snapshot(&self) -> D {
        D::from_base_session(self).await
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BaseSessionData {
    pub id: Ulid,
    pub user_id: i32,
    pub username: String,
    pub username_unicode: Option<String>,
    pub privileges: i32,
    pub created_at: DateTime<Utc>,
    pub last_active: u64,
}

#[async_trait]
impl FromBaseSession for BaseSessionData {
    async fn from_base_session(s: &BaseSession) -> Self {
        s.to_session_data()
    }
}

pub struct SessionFilter;

impl SessionFilter {
    #[inline]
    pub fn session_is_target(session: &BaseSession, to: &UserQuery) -> bool {
        match to {
            UserQuery::SessionId(t) => &session.id == t,
            UserQuery::UserId(t) => &session.user_id == t,
            UserQuery::Username(t) => session.username.load().as_ref() == t,
            UserQuery::UsernameUnicode(t) => session
                .username_unicode
                .load()
                .as_deref()
                .map(|n| n == t)
                .unwrap_or(false),
        }
    }
}
