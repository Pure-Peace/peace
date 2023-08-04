use crate::{BaseSession, UserKey};
use async_trait::async_trait;
use pb_bancho_state::UserQuery;
use peace_snapshot::CreateSnapshot;
use peace_unique_id::Ulid;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::Arc,
};
use tokio::sync::RwLock;
use tools::atomic::{AtomicOperation, AtomicValue, Usize};

#[derive(Debug)]
pub struct UserStore<T> {
    pub indexes: RwLock<UserIndexes<T>>,
    pub len: Usize,
}

impl<T> UserStore<T> {
    #[inline]
    pub fn new() -> Self {
        Self { indexes: RwLock::new(UserIndexes::new()), len: Usize::new(0) }
    }

    pub fn from_indexes(indexes: UserIndexes<T>) -> Self {
        let len = Usize::new(indexes.len());
        Self { indexes: RwLock::new(indexes), len }
    }

    #[inline]
    pub fn delete_inner(
        &self,
        indexes: &mut UserIndexes<T>,
        user_id: &i32,
        username: &str,
        session_id: &Ulid,
        username_unicode: Option<&str>,
    ) -> Option<Arc<T>> {
        let removed = indexes.remove_session(
            user_id,
            username,
            session_id,
            username_unicode,
        );

        if removed.is_some() {
            self.len.sub(1);
        }

        removed
    }

    #[inline]
    pub async fn get(&self, query: &UserQuery) -> Option<Arc<T>> {
        let indexes = self.indexes.read().await;
        Self::get_inner(&indexes, query)
    }

    #[inline]
    pub fn get_inner(
        indexes: &UserIndexes<T>,
        query: &UserQuery,
    ) -> Option<Arc<T>> {
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
        let indexes = self.indexes.read().await;
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

    #[inline]
    pub async fn clear(&self) {
        let mut indexes = self.indexes.write().await;
        indexes.user_id.clear();
        indexes.username.clear();
        indexes.username_unicode.clear();
        indexes.session_id.clear();

        self.len.set(0);
    }

    #[inline]
    pub fn length(&self) -> usize {
        self.len.val()
    }
}

impl<T> UserStore<T>
where
    T: Deref<Target = BaseSession>,
{
    #[inline]
    pub async fn create(&self, item: Arc<T>) -> Arc<T> {
        {
            let mut indexes = self.indexes.write().await;

            if let Some(prev) =
                Self::get_inner(&indexes, &UserQuery::UserId(item.user_id()))
            {
                self.delete_inner(
                    &mut indexes,
                    &prev.user_id,
                    &prev.username.load(),
                    &prev.id,
                    prev.username_unicode.load().as_deref().map(|s| s.as_str()),
                );
            }

            indexes.add_session(item.clone());
        };

        self.len.add(1);

        item
    }

    #[inline]
    pub async fn delete(&self, query: &UserQuery) -> Option<Arc<T>> {
        let mut indexes = self.indexes.write().await;

        let item = Self::get_inner(&indexes, query)?;

        self.delete_inner(
            &mut indexes,
            &item.user_id(),
            &item.username(),
            &item.session_id(),
            item.username_unicode().as_deref(),
        )
    }
}

#[async_trait]
impl<T, D> CreateSnapshot<Vec<D>> for UserStore<T>
where
    T: CreateSnapshot<D> + Sync + Send,
    D: Send,
{
    async fn create_snapshot(&self) -> Vec<D> {
        let mut sessions = Vec::with_capacity(self.length());
        for session in self.read().await.values() {
            sessions.push(session.create_snapshot().await);
        }

        sessions
    }
}

impl<T> Default for UserStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for UserStore<T> {
    type Target = RwLock<UserIndexes<T>>;

    fn deref(&self) -> &Self::Target {
        &self.indexes
    }
}

#[derive(Debug)]
pub struct UserIndexes<T> {
    pub session_id: BTreeMap<Ulid, Arc<T>>,
    pub user_id: BTreeMap<i32, Arc<T>>,
    pub username: HashMap<String, Arc<T>>,
    pub username_unicode: HashMap<String, Arc<T>>,
}

impl<T> UserIndexes<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            session_id: BTreeMap::new(),
            user_id: BTreeMap::new(),
            username: HashMap::new(),
            username_unicode: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            session_id: BTreeMap::new(),
            user_id: BTreeMap::new(),
            username: HashMap::with_capacity(capacity),
            username_unicode: HashMap::with_capacity(capacity),
        }
    }

    pub fn raw_add_session(
        &mut self,
        user_id: i32,
        username: String,
        session_id: Ulid,
        username_unicode: Option<String>,
        item: Arc<T>,
    ) {
        let username_unicode =
            username_unicode.unwrap_or_else(|| username.to_owned());

        self.session_id.insert(session_id, item.clone());
        self.user_id.insert(user_id, item.clone());
        self.username.insert(username, item.clone());
        self.username_unicode.insert(username_unicode, item);
    }

    pub fn remove_session(
        &mut self,
        user_id: &i32,
        username: &str,
        session_id: &Ulid,
        username_unicode: Option<&str>,
    ) -> Option<Arc<T>> {
        let mut removed = None;

        if let Some(s) = self.user_id.remove(user_id) {
            removed = Some(s);
        }
        if let Some(s) = self.username.remove(username) {
            removed = Some(s);
        }

        if let Some(s) = self.session_id.remove(session_id) {
            removed = Some(s);
        }

        if let Some(s) = username_unicode
            .and_then(|s| self.username_unicode.remove(s))
            .or_else(|| self.username_unicode.remove(username))
        {
            removed = Some(s);
        }

        removed
    }
}

impl<T> UserIndexes<T>
where
    T: Deref<Target = BaseSession>,
{
    pub fn add_session(&mut self, item: Arc<T>) {
        self.session_id.insert(item.session_id(), item.clone());
        self.user_id.insert(item.user_id(), item.clone());
        self.username.insert(item.username(), item.clone());
        self.username_unicode.insert(
            item.username_unicode().unwrap_or_else(|| item.username()),
            item,
        );
    }
}

impl<T> Default for UserIndexes<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for UserIndexes<T> {
    type Target = BTreeMap<i32, Arc<T>>;

    fn deref(&self) -> &Self::Target {
        &self.user_id
    }
}
