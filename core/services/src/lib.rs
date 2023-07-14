use peace_rpc::ParseError;

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

pub mod users {
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

                // Insert the user data into the relevant hash maps
                indexes.session_id.insert(item.session_id(), item.clone());
                indexes.user_id.insert(item.user_id(), item.clone());
                indexes.username.insert(item.username(), item.clone());
                indexes.username_unicode.insert(
                    item.username_unicode().unwrap_or_else(|| item.username()),
                    item.clone(),
                )
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

pub enum ServiceError<T> {
    Err(T),
    ParseError(ParseError),
}
