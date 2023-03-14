use crate::bancho_state::{
    DynUserSessionsService, Session, UserSessionsService,
};
use async_trait::async_trait;
use bancho_packets::PacketBuilder;
use peace_domain::bancho_state::CreateSessionDto;
use peace_pb::bancho_state::UserQuery;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::RwLock;
use tools::atomic::{AtomicOperation, AtomicValue, Usize};

#[derive(Debug, Default, Clone)]
pub struct SessionIndexes {
    /// A hash map that maps session IDs to user data
    pub session_id: HashMap<String, Arc<Session>>,
    /// A hash map that maps user IDs to user data
    pub user_id: HashMap<i32, Arc<Session>>,
    /// A hash map that maps usernames to user data
    pub username: HashMap<String, Arc<Session>>,
    /// A hash map that maps Unicode usernames to user data
    pub username_unicode: HashMap<String, Arc<Session>>,
}

impl Deref for SessionIndexes {
    type Target = HashMap<i32, Arc<Session>>;

    fn deref(&self) -> &Self::Target {
        &self.user_id
    }
}

/// A struct representing a collection of user sessions.
#[derive(Debug, Default)]
pub struct UserSessions {
    pub indexes: RwLock<SessionIndexes>,
    /// The number of user sessions in the collection
    pub len: Usize,
}

impl UserSessions {
    #[inline]
    pub async fn create(&self, session: Session) -> Arc<Session> {
        let session = Arc::new(session);

        let () = {
            let mut indexes = self.write().await;

            if let Some(old_session) = self
                .get_inner(&mut indexes, &UserQuery::UserId(session.user_id))
            {
                self.delete_inner(
                    &mut indexes,
                    &old_session.user_id,
                    &old_session.username.load(),
                    &old_session.id,
                    old_session
                        .username_unicode
                        .load()
                        .as_deref()
                        .map(|s| s.as_str()),
                );
            }

            // Insert the user data into the relevant hash maps
            indexes.session_id.insert(session.id.clone(), session.clone());
            indexes.user_id.insert(session.user_id, session.clone());
            indexes
                .username
                .insert(session.username.to_string(), session.clone());

            let _ = session
                .username_unicode
                .load()
                .as_ref()
                .and_then(|s| {
                    indexes
                        .username_unicode
                        .insert(s.to_string(), session.clone());

                    Some(())
                })
                .or_else(|| {
                    indexes
                        .username_unicode
                        .insert(session.username.to_string(), session.clone());

                    Some(())
                });
        };

        // Increment the length of the collection
        self.len.add_eq(1);

        // Return the session ID of the created or updated session
        session
    }

    #[inline]
    pub async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>> {
        let mut indexes = self.write().await;

        let session = self.get_inner(&indexes, query)?;

        self.delete_inner(
            &mut indexes,
            &session.user_id,
            &session.username.load(),
            &session.id,
            session.username_unicode.load().as_deref().map(|s| s.as_str()),
        )
    }

    #[inline]
    pub fn delete_inner(
        &self,
        indexes: &mut SessionIndexes,
        user_id: &i32,
        username: &str,
        session_id: &str,
        username_unicode: Option<&str>,
    ) -> Option<Arc<Session>> {
        let mut removed = None;

        indexes.user_id.remove(user_id).and_then(|s| Some(removed = Some(s)));
        indexes.username.remove(username).and_then(|s| Some(removed = Some(s)));

        indexes
            .session_id
            .remove(session_id)
            .and_then(|s| Some(removed = Some(s)));

        username_unicode
            .and_then(|s| indexes.username_unicode.remove(s))
            .or_else(|| indexes.username_unicode.remove(username))
            .and_then(|s| Some(removed = Some(s)));

        // Decrease the length of the map if a session was removed.
        if removed.is_some() {
            self.len.sub_eq(1);
        }

        removed
    }

    #[inline]
    pub async fn get(&self, query: &UserQuery) -> Option<Arc<Session>> {
        let indexes = self.read().await;
        self.get_inner(&indexes, query)
    }

    #[inline]
    pub fn get_inner(
        &self,
        indexes: &SessionIndexes,
        query: &UserQuery,
    ) -> Option<Arc<Session>> {
        match query {
            UserQuery::UserId(user_id) => indexes.user_id.get(user_id),
            UserQuery::Username(username) => indexes.username.get(username),
            UserQuery::UsernameUnicode(username_unicode) => {
                indexes.username_unicode.get(username_unicode)
            },
            UserQuery::SessionId(session_id) => {
                indexes.session_id.get(session_id)
            },
        }
        .cloned()
    }

    #[inline]
    pub async fn exists(&self, query: &UserQuery) -> bool {
        let indexes = self.read().await;
        match query {
            UserQuery::UserId(user_id) => indexes.user_id.contains_key(user_id),
            UserQuery::Username(username) => {
                indexes.username.contains_key(username)
            },
            UserQuery::UsernameUnicode(username_unicode) => {
                indexes.username_unicode.contains_key(username_unicode)
            },
            UserQuery::SessionId(session_id) => {
                indexes.session_id.contains_key(session_id)
            },
        }
    }

    /// Clears all sessions records from the [`UserSessions`].
    #[inline]
    pub async fn clear(&self) {
        let mut indexes = self.write().await;
        indexes.session_id.clear();
        indexes.username.clear();
        indexes.username_unicode.clear();
        indexes.session_id.clear();

        self.len.set(0);
    }

    /// Returns the number of sessions in the [`UserSessions`].
    #[inline]
    pub fn len(&self) -> usize {
        self.len.val()
    }
}

impl Deref for UserSessions {
    type Target = RwLock<SessionIndexes>;

    fn deref(&self) -> &Self::Target {
        &self.indexes
    }
}

#[derive(Debug, Default, Clone)]
pub struct UserSessionsServiceImpl {
    user_sessions: Arc<UserSessions>,
}

impl UserSessionsServiceImpl {
    #[inline]
    pub fn into_service(self) -> DynUserSessionsService {
        Arc::new(self) as DynUserSessionsService
    }
}

#[async_trait]
impl UserSessionsService for UserSessionsServiceImpl {
    #[inline]
    fn user_sessions(&self) -> &Arc<UserSessions> {
        &self.user_sessions
    }

    #[inline]
    async fn create(&self, create_session: CreateSessionDto) -> Arc<Session> {
        const LOG_TARGET: &str = "bancho_state::user_sessions::create_session";

        let session =
            self.user_sessions.create(Session::new(create_session)).await;

        let login_notify = Arc::new(
            PacketBuilder::from_batch([
                session.user_stats_packet(),
                session.user_presence_packet(),
            ])
            .build(),
        );

        let online_user_info = {
            let mut online_user_info = Vec::new();

            let user_sessions = self.user_sessions.read().await;

            for online_user in user_sessions.values() {
                online_user_info.extend(online_user.user_stats_packet());
                online_user_info.extend(online_user.user_presence_packet());

                online_user.push_packet(login_notify.clone()).await;
            }

            online_user_info
        };

        session.push_packet(online_user_info.into()).await;

        info!(
            target: LOG_TARGET,
            "Session created: {} [{}] ({})",
            session.username.load(),
            session.user_id,
            session.id
        );

        session
    }

    #[inline]
    async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>> {
        const LOG_TARGET: &str = "bancho_state::user_sessions::delete_session";

        let session = self.user_sessions.delete(query).await?;

        let logout_notify =
            Arc::new(bancho_packets::server::user_logout(session.user_id));

        let user_sessions = self.user_sessions.read().await;

        for session in user_sessions.values() {
            session.push_packet(logout_notify.clone()).await;
        }

        info!(
            target: LOG_TARGET,
            "Session deleted: {} [{}] ({})",
            session.username.load(),
            session.user_id,
            session.id
        );

        Some(session)
    }

    #[inline]
    async fn get(&self, query: &UserQuery) -> Option<Arc<Session>> {
        self.user_sessions.get(query).await
    }

    #[inline]
    async fn exists(&self, query: &UserQuery) -> bool {
        self.user_sessions.exists(query).await
    }

    #[inline]
    async fn clear(&self) {
        self.user_sessions.clear().await
    }

    #[inline]
    fn len(&self) -> usize {
        self.user_sessions.len()
    }
}
