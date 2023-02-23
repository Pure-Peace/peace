use std::{collections::HashMap, ops::Deref, sync::Arc};

use chrono::{DateTime, Utc};
use peace_pb::services::bancho_state_rpc::{
    ConnectionInfo, CreateUserSessionRequest, UserQuery,
};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct User {
    pub session_id: String,
    pub user_id: i32,
    pub username: String,
    pub username_unicode: Option<String>,
    pub privileges: i32,
    pub connection_info: ConnectionInfo,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl User {
    #[inline]
    pub fn update_active(&mut self) {
        self.last_active = Utc::now();
    }
}

impl From<CreateUserSessionRequest> for User {
    #[inline]
    fn from(res: CreateUserSessionRequest) -> Self {
        let CreateUserSessionRequest {
            user_id,
            username,
            username_unicode,
            privileges,
            connection_info,
        } = res;

        Self {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            username,
            username_unicode,
            privileges,
            connection_info: connection_info.unwrap(),
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct UserSessions {
    pub indexed_by_session_id: HashMap<String, Arc<RwLock<User>>>,
    pub indexed_by_user_id: HashMap<i32, Arc<RwLock<User>>>,
    pub indexed_by_username: HashMap<String, Arc<RwLock<User>>>,
    pub indexed_by_username_unicode: HashMap<String, Arc<RwLock<User>>>,
    pub len: usize,
}

impl UserSessions {
    #[inline]
    pub async fn create(&mut self, user: User) -> String {
        self.delete(&UserQuery::UserId(user.user_id)).await;

        let (session_id, user_id, username, username_unicode) = (
            user.session_id.clone(),
            user.user_id,
            user.username.clone(),
            user.username_unicode.clone(),
        );

        let ptr = Arc::new(RwLock::new(user));

        self.indexed_by_session_id.insert(session_id.clone(), ptr.clone());
        self.indexed_by_user_id.insert(user_id, ptr.clone());
        self.indexed_by_username.insert(username, ptr.clone());

        username_unicode
            .and_then(|s| self.indexed_by_username_unicode.insert(s, ptr));

        self.len += 1;

        session_id
    }

    #[inline]
    pub async fn delete(
        &mut self,
        query: &UserQuery,
    ) -> Option<Arc<RwLock<User>>> {
        self.delete_user(self.get(query)?.write().await)
    }

    #[inline]
    pub fn delete_user(
        &mut self,
        user: impl Deref<Target = User>,
    ) -> Option<Arc<RwLock<User>>> {
        let mut removed = None;

        self.indexed_by_user_id
            .remove(&user.user_id)
            .and_then(|u| Some(removed = Some(u)));
        self.indexed_by_username
            .remove(&user.username)
            .and_then(|u| Some(removed = Some(u)));
        self.indexed_by_session_id
            .remove(&user.session_id)
            .and_then(|u| Some(removed = Some(u)));

        user.username_unicode
            .as_ref()
            .and_then(|s| self.indexed_by_username_unicode.remove(s))
            .and_then(|u| Some(removed = Some(u)));

        if removed.is_some() {
            self.len -= 1;
        }

        removed
    }

    #[inline]
    pub fn get(&self, query: &UserQuery) -> Option<Arc<RwLock<User>>> {
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

    #[inline]
    pub fn clear(&mut self) {
        self.indexed_by_session_id.clear();
        self.indexed_by_username.clear();
        self.indexed_by_username_unicode.clear();
        self.indexed_by_session_id.clear();

        self.len = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}
