use std::path::Path;

use async_trait::async_trait;
use peace_rpc_error::{RpcError, TonicError};
use tonic::Status;

#[macro_use]
extern crate peace_logs;

#[allow(unused_imports)]
#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

pub mod bancho;
pub mod bancho_state;
pub mod chat;
pub mod gateway;
pub mod geoip;
pub mod signature;

pub mod rpc_config {
    use clap_serde_derive::ClapSerde;
    use peace_api::define_rpc_client_config;
    use peace_pb::*;

    define_rpc_client_config!(
        service_name: bancho,
        config_name: BanchoRpcConfig,
        default_uri: "http://127.0.0.1:5010"
    );

    define_rpc_client_config!(
        service_name: bancho_state,
        config_name: BanchoStateRpcConfig,
        default_uri: "http://127.0.0.1:5011"
    );

    define_rpc_client_config!(
        service_name: chat,
        config_name: ChatRpcConfig,
        default_uri: "http://127.0.0.1:5012"
    );

    define_rpc_client_config!(
        service_name: geoip,
        config_name: GeoipRpcConfig,
        default_uri: "http://127.0.0.1:5013"
    );

    define_rpc_client_config!(
        service_name: signature,
        config_name: SignatureRpcConfig,
        default_uri: "http://127.0.0.1:5014"
    );
}

pub trait FromRpcClient: RpcClient {
    fn from_client(client: Self::Client) -> Self;
}

pub trait RpcClient {
    type Client;

    fn client(&self) -> Self::Client;
}

pub trait IntoService<T>: Sized + Sync + Send + 'static {
    fn into_service(self) -> T;
}

pub trait DumpConfig {
    fn dump_path(&self) -> &str;
    fn save_dump(&self) -> bool;
    fn load_dump(&self) -> bool;
    fn dump_expries(&self) -> u64;
}

#[async_trait]
pub trait DumpData<D> {
    async fn dump_data(&self) -> D;
}

#[async_trait]
pub trait DumpToDisk<D> {
    async fn dump_to_disk(&self, path: &str) -> Result<usize, DumpError>;
}

#[async_trait]
pub trait TryDumpToDisk {
    async fn try_dump_to_disk(
        &self,
        dump_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait]
impl<T, D> DumpToDisk<D> for T
where
    T: DumpData<D> + Sync + Send,
    D: serde::Serialize + Send,
{
    async fn dump_to_disk(&self, path: &str) -> Result<usize, DumpError> {
        let dump_data = self.dump_data().await;

        let bytes_data = bincode::serialize(&dump_data)
            .map_err(|err| DumpError::SerializeError(err.to_string()))?;

        let path = Path::new(path);

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| DumpError::CreateDirError(err.to_string()))?;
        }

        tokio::fs::write(path, &bytes_data)
            .await
            .map_err(|err| DumpError::WriteFileError(err.to_string()))?;

        Ok(bytes_data.len())
    }
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum DumpError {
    #[error("SerializeError: {0}")]
    SerializeError(String),
    #[error("CreateDirError: {0}")]
    CreateDirError(String),
    #[error("WriteFileError: {0}")]
    WriteFileError(String),
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for DumpError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}

pub mod users {
    use crate::DumpData;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use peace_domain::bancho_state::CreateSessionDto;
    use peace_pb::bancho_state::UserQuery;
    use std::{
        collections::{BTreeMap, HashMap},
        ops::Deref,
        sync::Arc,
    };
    use tokio::sync::RwLock;
    use tools::{
        atomic::{
            Atomic, AtomicOperation, AtomicOption, AtomicValue, Usize, I32, U64,
        },
        Timestamp, Ulid,
    };

    #[async_trait]
    pub trait FromSession<E> {
        async fn from_session(s: &Session<E>) -> Self;
    }

    #[derive(Debug, Serialize)]
    pub struct Session<T> {
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
        pub extends: T,
    }

    impl<T, D> From<SessionData<D>> for Session<T>
    where
        T: From<D>,
    {
        fn from(u: SessionData<D>) -> Self {
            Self {
                id: u.id,
                user_id: u.user_id,
                username: u.username.into(),
                username_unicode: u.username_unicode.into(),
                privileges: u.privileges.into(),
                created_at: u.created_at,
                last_active: u.last_active.into(),
                extends: u.extends.into(),
            }
        }
    }

    impl<T> UserKey for Session<T> {
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

    impl<T> Session<T> {
        #[inline]
        pub fn new(create_session: CreateSessionDto<T>) -> Self {
            let CreateSessionDto {
                user_id,
                username,
                username_unicode,
                privileges,
                extends,
            } = create_session;

            Self {
                id: Ulid::new(),
                user_id,
                username: username.into(),
                username_unicode: username_unicode.into(),
                privileges: privileges.into(),
                created_at: Utc::now(),
                last_active: Timestamp::now().into(),
                extends,
            }
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
        pub fn is_deactive(
            &self,
            current_timestamp: u64,
            deadline: u64,
        ) -> bool {
            current_timestamp > self.last_active.val() + deadline
        }

        #[inline]
        pub fn update_active(&self) {
            self.last_active.set(Timestamp::now());
        }
    }

    #[async_trait]
    impl<T, D> DumpData<D> for Session<T>
    where
        T: Sync + Send,
        D: FromSession<T>,
    {
        async fn dump_data(&self) -> D {
            D::from_session(self).await
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SessionData<D> {
        pub id: Ulid,
        pub user_id: i32,
        pub username: String,
        pub username_unicode: Option<String>,
        pub privileges: i32,
        pub created_at: DateTime<Utc>,
        pub last_active: u64,
        pub extends: D,
    }

    #[async_trait]
    impl<T, D> FromSession<T> for SessionData<D>
    where
        T: DumpData<D> + Sync + Send,
    {
        async fn from_session(s: &Session<T>) -> Self {
            Self {
                id: s.id,
                user_id: s.user_id,
                username: s.username(),
                username_unicode: s.username_unicode(),
                privileges: s.privileges.val(),
                created_at: s.created_at,
                last_active: s.last_active.val(),
                extends: s.extends.dump_data().await,
            }
        }
    }

    pub trait UserKey {
        fn session_id(&self) -> Ulid;
        fn user_id(&self) -> i32;
        fn username(&self) -> String;
        fn username_unicode(&self) -> Option<String>;
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

        pub fn add_session(&mut self, item: Arc<T>)
        where
            T: UserKey,
        {
            self.session_id.insert(item.session_id(), item.clone());
            self.user_id.insert(item.user_id(), item.clone());
            self.username.insert(item.username(), item.clone());
            self.username_unicode.insert(
                item.username_unicode().unwrap_or_else(|| item.username()),
                item,
            );
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

    #[derive(Debug)]
    pub struct UserStore<T> {
        pub indexes: RwLock<UserIndexes<T>>,
        pub len: Usize,
    }

    impl<T> UserStore<T>
    where
        T: UserKey,
    {
        #[inline]
        pub fn new() -> Self {
            Self {
                indexes: RwLock::new(UserIndexes::new()),
                len: Usize::new(0),
            }
        }

        pub fn from_indexes(indexes: UserIndexes<T>) -> Self {
            let len = Usize::new(indexes.len());
            Self { indexes: RwLock::new(indexes), len }
        }

        #[inline]
        pub async fn create(&self, item: T) -> Arc<T> {
            let item = Arc::new(item);

            {
                let mut indexes = self.indexes.write().await;

                if let Some(prev) = Self::get_inner(
                    &indexes,
                    &UserQuery::UserId(item.user_id()),
                ) {
                    self.delete_inner(
                        &mut indexes,
                        &prev.user_id(),
                        &prev.username(),
                        &prev.session_id(),
                        prev.username_unicode().as_deref(),
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
                UserQuery::UserId(user_id) => {
                    indexes.user_id.contains_key(user_id)
                },
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

    impl<T> UserStore<Session<T>> {
        pub async fn dump_sessions<D>(&self) -> Vec<D>
        where
            D: FromSession<T>,
        {
            let mut sessions = Vec::with_capacity(self.length());
            for session in self.read().await.values() {
                sessions.push(D::from_session(session.as_ref()).await);
            }

            sessions
        }
    }

    impl<T> Default for UserStore<T>
    where
        T: UserKey,
    {
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

    pub struct SessionFilter;

    impl SessionFilter {
        #[inline]
        pub fn session_is_target<T>(
            session: &Session<T>,
            to: &UserQuery,
        ) -> bool {
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
}
