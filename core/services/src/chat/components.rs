use crate::bancho_state::{MessageQueue, Session, UserStore};
use bitmask_enum::bitmask;
use chrono::{DateTime, Utc};
use derive_deref::Deref;
use peace_pb::chat::{ChannelInfo, ChannelSessionsCounter};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Deref,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};
use tools::atomic::{Atomic, AtomicOperation, AtomicOption, AtomicValue, U64};

pub type UserSessions = UserStore<Session<()>>;

#[derive(
    Debug, Copy, Clone, Default, Primitive, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum ChannelType {
    #[default]
    Private = 0,
    Public = 1,
    Group = 2,
    Multiplayer = 3,
    Spectaor = 4,
}

#[derive(Default)]
#[bitmask(i32)]
pub enum Platform {
    #[default]
    None = 0,
    Bancho = 1,
    Lazer = 2,
    Web = 3,
}

#[derive(Debug, Default, Clone)]
pub struct OnlinePlatforms(Arc<Atomic<Platform>>);

impl Deref for OnlinePlatforms {
    type Target = Arc<Atomic<Platform>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl OnlinePlatforms {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.0.load().bits() == 0
    }

    #[inline]
    pub fn val(&self) -> i32 {
        self.0.load().bits()
    }

    #[inline]
    pub fn add(&self, platforms: Platform) {
        self.0.set(
            Platform::from(self.0.load().bits() | platforms.bits()).into(),
        );
    }

    #[inline]
    pub fn remove(&self, platforms: Platform) {
        self.0.set(
            Platform::from(self.0.load().bits() & !platforms.bits()).into(),
        );
    }
}

#[derive(Debug, Clone, Copy, Deref)]
pub struct BanchoChannelSession(i32);

#[derive(Debug, Clone, Copy, Deref)]
pub struct LazerChannelSession(i32);

#[derive(Debug, Clone, Copy, Deref)]
pub struct WebChannelSession(i32);

#[derive(Debug, Clone)]
pub struct UnspecifiedChannelSession {
    pub user_id: i32,
    pub online_platforms: OnlinePlatforms,
}

impl UnspecifiedChannelSession {
    #[inline]
    pub fn new(user_id: i32) -> Self {
        Self { user_id, online_platforms: OnlinePlatforms::default() }
    }
}

#[derive(Debug, Default)]
pub struct SessionCounter {
    pub unspecified: U64,
    pub platform_bancho: U64,
    pub platform_lazer: U64,
    pub platform_web: U64,
}

impl SessionCounter {
    #[inline]
    pub fn counter(&self, platforms: Platform) -> &'_ U64 {
        if platforms.contains(Platform::Bancho) {
            return &self.platform_bancho;
        }

        if platforms.contains(Platform::Lazer) {
            return &self.platform_lazer;
        }

        if platforms.contains(Platform::Web) {
            return &self.platform_web;
        }

        &self.unspecified
    }

    #[inline]
    pub fn value(&self, platforms: Platform) -> u64 {
        self.counter(platforms).val()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChannelSessionIndexes {
    pub unspecified: HashMap<i32, Arc<UnspecifiedChannelSession>>,
    pub bancho: HashMap<i32, Arc<BanchoChannelSession>>,
    pub lazer: HashMap<i32, Arc<LazerChannelSession>>,
    pub web: HashMap<i32, Arc<WebChannelSession>>,
}

#[derive(Debug, Default)]
pub struct ChannelSessions {
    pub indexes: RwLock<ChannelSessionIndexes>,
    pub counter: SessionCounter,
}

impl ChannelSessions {
    #[inline]
    pub fn new(users: Vec<i32>) -> Self {
        let counter = SessionCounter::default();
        counter.unspecified.add(users.len() as u64);

        let indexes = ChannelSessionIndexes {
            unspecified: users
                .into_iter()
                .map(|user_id| {
                    (user_id, UnspecifiedChannelSession::new(user_id).into())
                })
                .collect(),
            ..Default::default()
        }
        .into();

        Self { indexes, counter }
    }

    #[inline]
    pub async fn add_user(&self, user_id: i32, platforms: Platform) {
        let mut indexes = self.indexes.write().await;

        if platforms.is_none() {
            if let Entry::Vacant(e) = indexes.unspecified.entry(user_id) {
                e.insert(UnspecifiedChannelSession::new(user_id).into());
                self.counter.unspecified.add(1);
            }
        } else {
            match indexes.unspecified.get(&user_id) {
                Some(unspecified_session) => {
                    unspecified_session.online_platforms.add(platforms);
                },
                None => {
                    let unspecified_session =
                        UnspecifiedChannelSession::new(user_id);
                    unspecified_session.online_platforms.add(platforms);

                    indexes
                        .unspecified
                        .insert(user_id, unspecified_session.into());
                },
            };

            if platforms.contains(Platform::Bancho) {
                if let Entry::Vacant(e) = indexes.bancho.entry(user_id) {
                    e.insert(BanchoChannelSession(user_id).into());
                    self.counter.platform_bancho.add(1);
                }
            }

            if platforms.contains(Platform::Lazer) {
                if let Entry::Vacant(e) = indexes.lazer.entry(user_id) {
                    e.insert(LazerChannelSession(user_id).into());
                    self.counter.platform_lazer.add(1);
                }
            }

            if platforms.contains(Platform::Web) {
                if let Entry::Vacant(e) = indexes.web.entry(user_id) {
                    e.insert(WebChannelSession(user_id).into());
                    self.counter.platform_web.add(1);
                }
            }
        }
    }

    #[inline]
    pub async fn remove_user(&self, user_id: &i32) {
        let mut indexes = self.indexes.write().await;

        indexes
            .unspecified
            .remove(user_id)
            .map(|_| self.counter.unspecified.sub(1));
        indexes
            .bancho
            .remove(user_id)
            .map(|_| self.counter.platform_bancho.sub(1));
        indexes
            .lazer
            .remove(user_id)
            .map(|_| self.counter.platform_lazer.sub(1));
        indexes.web.remove(user_id).map(|_| self.counter.platform_web.sub(1));
    }

    #[inline]
    pub async fn remove_user_platforms(
        &self,
        user_id: &i32,
        platforms: Platform,
    ) {
        let mut indexes = self.indexes.write().await;

        if platforms.is_none() {
            let mut indexes = self.indexes.write().await;

            if let Some(session) = indexes.unspecified.get(user_id) {
                session.online_platforms.set(Default::default())
            }
            indexes
                .bancho
                .remove(user_id)
                .map(|_| self.counter.platform_bancho.sub(1));
            indexes
                .lazer
                .remove(user_id)
                .map(|_| self.counter.platform_lazer.sub(1));
            indexes
                .web
                .remove(user_id)
                .map(|_| self.counter.platform_web.sub(1));
        } else {
            match indexes.unspecified.get(user_id) {
                Some(unspecified_session) => {
                    unspecified_session.online_platforms.remove(platforms);
                },
                None => return,
            };

            if platforms.contains(Platform::Bancho) {
                indexes
                    .bancho
                    .remove(user_id)
                    .map(|_| self.counter.platform_bancho.sub(1));
            }

            if platforms.contains(Platform::Lazer) {
                indexes
                    .lazer
                    .remove(user_id)
                    .map(|_| self.counter.platform_lazer.sub(1));
            }

            if platforms.contains(Platform::Web) {
                indexes
                    .web
                    .remove(user_id)
                    .map(|_| self.counter.platform_web.sub(1));
            }
        }
    }

    #[inline]
    pub async fn user_exists(
        &self,
        user_id: &i32,
        platforms: Platform,
    ) -> bool {
        let indexes = self.indexes.read().await;

        if platforms.is_none() {
            return indexes.unspecified.contains_key(user_id);
        }

        platforms.contains(Platform::Bancho)
            && indexes.bancho.contains_key(user_id)
            || platforms.contains(Platform::Lazer)
                && indexes.lazer.contains_key(user_id)
            || platforms.contains(Platform::Web)
                && indexes.web.contains_key(user_id)
    }
}

#[derive(Debug, Default)]
pub struct ChannelMetadata {
    pub id: u64,
    pub name: Atomic<String>,
    pub channel_type: ChannelType,
    pub description: AtomicOption<String>,
}

#[derive(Default)]
pub struct Channel {
    pub metadata: ChannelMetadata,
    pub sessions: ChannelSessions,
    pub message_queue: Arc<Mutex<MessageQueue>>,
    pub created_at: DateTime<Utc>,
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel")
            .field("metadata", &self.metadata)
            .field("sessions", &self.sessions)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl Deref for Channel {
    type Target = ChannelMetadata;

    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl Channel {
    #[inline]
    pub fn new(
        metadata: ChannelMetadata,
        sessions: Option<ChannelSessions>,
    ) -> Self {
        Self {
            metadata,
            sessions: sessions.unwrap_or_default(),
            message_queue: Arc::new(Mutex::new(MessageQueue::default())),
            created_at: Utc::now(),
        }
    }

    #[inline]
    pub fn session_count(&self, platforms: Platform) -> u64 {
        self.sessions.counter.value(platforms)
    }

    #[inline]
    pub fn channel_info(&self) -> ChannelInfo {
        ChannelInfo {
            id: self.id,
            name: self.name.to_string(),
            channel_type: self.channel_type as i32,
            description: self
                .description
                .load()
                .as_ref()
                .map(|s| s.to_string()),
            counter: Some(ChannelSessionsCounter {
                unspecified: self.session_count(Platform::None),
                bancho: self.session_count(Platform::Bancho),
                lazer: self.session_count(Platform::Lazer),
                web: self.session_count(Platform::Web),
            }),
            users: None,
        }
    }
}
