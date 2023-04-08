use crate::bancho_state::{
    DynUserSessionsService, Packet, Session, UserSessionsService,
};
use async_trait::async_trait;
use bancho_packets::server::UserLogout;
use peace_domain::bancho_state::CreateSessionDto;
use peace_pb::bancho_state::UserQuery;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};
use tools::{
    atomic::{AtomicOperation, AtomicValue, Usize},
    Ulid,
};

#[derive(Debug, Default)]
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
            indexes.session_id.insert(session.id.clone(), session.clone());
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
        session_id: &str,
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

pub type Validator = Arc<dyn Fn() -> bool + Sync + Send + 'static>;

#[derive(Clone)]
pub struct Message<T: Clone, K> {
    pub content: T,
    pub has_read: HashSet<K>,
    pub validator: Option<Validator>,
}

#[derive(Clone)]
pub struct ReceivedMessages<T: Clone> {
    pub messages: Vec<T>,
    pub last_msg_id: Ulid,
}

#[derive(Clone, Default)]
pub struct Queue<T: Clone, K: Clone + Eq + Hash> {
    pub messsages: BTreeMap<Ulid, Message<T, K>>,
}

impl<T: Clone, K: Clone + Eq + Hash> Queue<T, K> {
    #[inline]
    pub fn push(&mut self, content: T, validator: Option<Validator>) {
        self.messsages.insert(
            Ulid::generate(),
            Message { content, has_read: HashSet::default(), validator },
        );
    }

    #[inline]
    pub fn push_excludes(
        &mut self,
        content: T,
        excludes: impl IntoIterator<Item = K>,
        validator: Option<Validator>,
    ) {
        self.messsages.insert(
            Ulid::generate(),
            Message {
                content,
                has_read: HashSet::from_iter(excludes),
                validator,
            },
        );
    }

    #[inline]
    pub fn receive(
        &mut self,
        read_key: &K,
        start_msg_id: &Ulid,
    ) -> Option<ReceivedMessages<T>> {
        let mut should_delete = None::<Vec<Ulid>>;
        let mut messages = None::<Vec<T>>;
        let mut last_msg_id = None::<Ulid>;

        for (msg_id, msg) in self.messsages.range_mut(start_msg_id..) {
            if msg.has_read.contains(read_key) {
                continue;
            }

            if let Some(valid) = &msg.validator {
                if !valid() {
                    match should_delete {
                        Some(ref mut should_delete) => {
                            should_delete.push(*msg_id)
                        },
                        None => should_delete = Some(vec![*msg_id]),
                    }
                    continue;
                }
            }

            match messages {
                Some(ref mut messages) => messages.push(msg.content.clone()),
                None => messages = Some(vec![msg.content.clone()]),
            }
            msg.has_read.insert(read_key.clone());
            last_msg_id = Some(*msg_id);
        }

        should_delete.map(|list| {
            list.into_iter().map(|msg_id| self.messsages.remove(&msg_id))
        });

        messages.map(|messages| ReceivedMessages {
            messages,
            last_msg_id: last_msg_id.unwrap(),
        })
    }
}

#[derive(Clone)]
pub struct UserSessionsServiceImpl {
    user_sessions: Arc<UserSessions>,
    notify_queue: Arc<Mutex<Queue<Packet, i32>>>,
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
            notify_queue: Arc::new(Mutex::new(Queue {
                messsages: BTreeMap::new(),
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
    fn notify_queue(&self) -> &Arc<Mutex<Queue<Packet, i32>>> {
        &self.notify_queue
    }

    #[inline]
    async fn create(&self, create_session: CreateSessionDto) -> Arc<Session> {
        const LOG_TARGET: &str = "bancho_state::user_sessions::create_session";

        let session =
            self.user_sessions.create(Session::new(create_session)).await;

        let weak = Arc::downgrade(&session);

        self.notify_queue.lock().await.push_excludes(
            session.user_info_packets().into(),
            [session.user_id],
            Some(Arc::new(move || weak.upgrade().is_some())),
        );

        session
            .enqueue_packets(
                self.user_sessions
                    .read()
                    .await
                    .values()
                    .map(|session| Packet::Data(session.user_info_packets())),
            )
            .await;
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
