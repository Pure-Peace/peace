use crate::bancho_state::{
    DynUserSessionsService, Packet, Session, UserSessionsService,
    PRESENCE_SHARD,
};
use async_trait::async_trait;
use bancho_packets::server::UserLogout;
use peace_domain::bancho_state::CreateSessionDto;
use peace_pb::bancho_state::UserQuery;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};
use tools::{
    atomic::{AtomicOperation, AtomicValue, Usize},
    message_queue::MessageQueue,
    Ulid,
};

#[derive(Clone)]
pub struct UserSessionsServiceImpl {
    pub user_sessions: Arc<UserSessions>,
    pub notify_queue: Arc<Mutex<MessageQueue<Packet, i32, Ulid>>>,
}

impl UserSessionsServiceImpl {
    #[inline]
    pub fn into_service(self) -> DynUserSessionsService {
        Arc::new(self) as DynUserSessionsService
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            user_sessions: Arc::default(),
            notify_queue: Arc::new(Mutex::new(MessageQueue {
                messages: BTreeMap::new(),
            })),
        }
    }
}

#[async_trait]
impl UserSessionsService for UserSessionsServiceImpl {
    #[inline]
    fn user_sessions(&self) -> &Arc<UserSessions> {
        &self.user_sessions
    }

    #[inline]
    fn notify_queue(&self) -> &Arc<Mutex<MessageQueue<Packet, i32, Ulid>>> {
        &self.notify_queue
    }

    #[inline]
    async fn create(&self, create_session: CreateSessionDto) -> Arc<Session> {
        const LOG_TARGET: &str = "bancho_state::user_sessions::create_session";

        let session =
            self.user_sessions.create(Session::new(create_session)).await;

        let weak = Arc::downgrade(&session);

        self.notify_queue.lock().await.push_excludes(
            bancho_packets::server::UserPresenceSingle::pack(session.user_id)
                .into(),
            [session.user_id],
            Some(Arc::new(move |_| weak.upgrade().is_some())),
        );

        let online_users = {
            self.user_sessions
                .read()
                .await
                .keys()
                .copied()
                .collect::<Vec<i32>>()
        };

        let online_users_len = online_users.len();

        let mut pending_packets = Vec::with_capacity(
            online_users_len / PRESENCE_SHARD
                + if (online_users_len % PRESENCE_SHARD) > 0 { 2 } else { 1 },
        );

        pending_packets.push(Packet::Data(session.user_info_packets()));

        for shard in online_users.chunks(PRESENCE_SHARD) {
            pending_packets.push(Packet::Data(
                bancho_packets::server::UserPresenceBundle::pack(shard),
            ))
        }

        session.enqueue_packets(pending_packets).await;

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

        self.notify_queue
            .lock()
            .await
            .push(UserLogout::pack(session.user_id).into(), None);

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

#[derive(Debug, Default)]
pub struct SessionIndexes {
    /// A hash map that maps session IDs to user data
    pub session_id: BTreeMap<Ulid, Arc<Session>>,
    /// A hash map that maps user IDs to user data
    pub user_id: BTreeMap<i32, Arc<Session>>,
    /// A hash map that maps usernames to user data
    pub username: HashMap<String, Arc<Session>>,
    /// A hash map that maps Unicode usernames to user data
    pub username_unicode: HashMap<String, Arc<Session>>,
}

impl Deref for SessionIndexes {
    type Target = BTreeMap<i32, Arc<Session>>;

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

        {
            let mut indexes = self.write().await;

            if let Some(old_session) =
                Self::get_inner(&indexes, &UserQuery::UserId(session.user_id))
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
            indexes.session_id.insert(session.id, session.clone());
            indexes.user_id.insert(session.user_id, session.clone());
            indexes
                .username
                .insert(session.username.to_string(), session.clone());

            let _ = session
                .username_unicode
                .load()
                .as_ref()
                .map(|s| {
                    indexes
                        .username_unicode
                        .insert(s.to_string(), session.clone());
                })
                .or_else(|| {
                    indexes
                        .username_unicode
                        .insert(session.username.to_string(), session.clone());

                    Some(())
                });
        };

        // Increment the length of the collection
        self.len.add(1);

        // Return the session ID of the created or updated session
        session
    }

    #[inline]
    pub async fn delete(&self, query: &UserQuery) -> Option<Arc<Session>> {
        let mut indexes = self.write().await;

        let session = Self::get_inner(&indexes, query)?;

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
        session_id: &Ulid,
        username_unicode: Option<&str>,
    ) -> Option<Arc<Session>> {
        let mut removed = None;

        if let Some(s) = indexes.user_id.remove(user_id) {
            removed = Some(s);
        }
        if let Some(s) = indexes.username.remove(username) {
            removed = Some(s);
        }

        if let Some(s) = indexes.session_id.remove(session_id) {
            removed = Some(s);
        }

        if let Some(s) = username_unicode
            .and_then(|s| indexes.username_unicode.remove(s))
            .or_else(|| indexes.username_unicode.remove(username))
        {
            removed = Some(s);
        }

        // Decrease the length of the map if a session was removed.
        if removed.is_some() {
            self.len.sub(1);
        }

        removed
    }

    #[inline]
    pub async fn get(&self, query: &UserQuery) -> Option<Arc<Session>> {
        let indexes = self.read().await;
        Self::get_inner(&indexes, query)
    }

    #[inline]
    pub fn get_inner(
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
