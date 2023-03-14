use crate::bancho_state::{
    DynUserSessionsService, Session, UserSessionsService,
};
use async_trait::async_trait;
use peace_pb::bancho_state::UserQuery;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::RwLock;

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

        let _ = session
            .username_unicode
            .load()
            .as_ref()
            .and_then(|s| {
                self.indexed_by_username_unicode
                    .insert(s.to_string(), session.clone());

                Some(())
            })
            .or_else(|| {
                self.indexed_by_username_unicode
                    .insert(session.username.to_string(), session.clone());

                Some(())
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
            .or_else(|| self.indexed_by_username_unicode.remove(username))
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

impl Deref for UserSessionsInner {
    type Target = HashMap<i32, Arc<Session>>;

    fn deref(&self) -> &Self::Target {
        &self.indexed_by_user_id
    }
}

#[derive(Debug, Default, Clone)]
pub struct UserSessionsServiceImpl {
    user_sessions: Arc<RwLock<UserSessionsInner>>,
}

impl UserSessionsServiceImpl {
    pub fn into_service(self) -> DynUserSessionsService {
        Arc::new(self) as DynUserSessionsService
    }
}

#[async_trait]
impl UserSessionsService for UserSessionsServiceImpl {
    fn user_sessions(&self) -> &Arc<RwLock<UserSessionsInner>> {
        &self.user_sessions
    }

    async fn create(&self, session: Session) -> Arc<Session> {
        self.user_sessions.write().await.create(session).await
    }

    async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>> {
        let session = self.user_sessions.write().await.delete(query).await?;

        let logout_notify =
            Arc::new(bancho_packets::server::user_logout(session.user_id));

        let user_sessions = self.user_sessions.read().await;

        for session in user_sessions.values() {
            session.push_packet(logout_notify.clone()).await;
        }

        Some(session)
    }

    async fn get(&self, query: &UserQuery) -> Option<Arc<Session>> {
        self.user_sessions.read().await.get(query)
    }

    async fn exists(&self, query: &UserQuery) -> bool {
        self.user_sessions.read().await.exists(query)
    }

    async fn clear(&self) {
        self.user_sessions.write().await.clear()
    }

    async fn len(&self) -> usize {
        self.user_sessions.read().await.len()
    }
}
