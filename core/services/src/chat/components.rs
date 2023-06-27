use chrono::{DateTime, Utc};
use derive_deref::Deref;
use peace_pb::chat::{ChannelInfo, ChannelSessionsCounter};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};
use tokio::sync::RwLock;
use tools::atomic::{Atomic, AtomicOperation, AtomicOption, AtomicValue, U64};

#[derive(
    Debug, Copy, Clone, Default, PartialEq, Primitive, Serialize, Deserialize,
)]
pub enum ChannelType {
    #[default]
    Private = 0,
    Public = 1,
    Group = 2,
    Multiplayer = 3,
    Spectaor = 4,
}

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    Primitive,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum Platform {
    #[default]
    Bancho = 0,
    Lazer = 1,
    Web = 2,
}

pub struct PlatformsLoader;

impl PlatformsLoader {
    #[inline]
    pub fn load_from_vec(platforms: Vec<i32>) -> Vec<Platform> {
        platforms
            .into_iter()
            .map(Platform::try_from)
            .filter(|result| {
                if result.is_err() {
                    warn!("Unsupported Platform: {:?}", result)
                }
                true
            })
            .map(|p| p.unwrap())
            .collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct OnlinePlatforms(Arc<Atomic<HashSet<Platform>>>);

impl Deref for OnlinePlatforms {
    type Target = Arc<Atomic<HashSet<Platform>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl OnlinePlatforms {
    #[inline]
    pub fn from_iter(platforms: impl IntoIterator<Item = Platform>) -> Self {
        Self(Atomic::new(HashSet::from_iter(platforms)).into())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.load().is_empty()
    }

    #[inline]
    pub fn val(&self) -> Option<Vec<Platform>> {
        let platforms = self.0.load();
        if platforms.is_empty() {
            return None;
        }

        Some(platforms.iter().map(|p| p.to_owned()).collect())
    }

    #[inline]
    pub fn add(&self, platforms: &[Platform]) {
        let mut pre_val = self.clone_inner();
        pre_val.extend(platforms.iter());

        self.0.set(pre_val.into())
    }

    #[inline]
    pub fn remove(&self, platforms: &[Platform]) {
        let mut pre_val = self.clone_inner();
        for platform in platforms {
            pre_val.remove(platform);
        }

        self.0.set(pre_val.into())
    }

    #[inline]
    pub fn clone_inner(&self) -> HashSet<Platform> {
        self.0.load().as_ref().clone()
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
    pub fn counter(&self, platform: Option<&Platform>) -> &'_ U64 {
        if let Some(platform) = platform {
            match platform {
                &Platform::Bancho => &self.platform_bancho,
                &Platform::Lazer => &self.platform_lazer,
                &Platform::Web => &self.platform_web,
            }
        } else {
            &self.unspecified
        }
    }

    #[inline]
    pub fn value(&self, platform: Option<&Platform>) -> u64 {
        self.counter(platform).val()
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
    pub async fn add_user(
        &self,
        user_id: i32,
        additional_platforms: Option<Vec<Platform>>,
    ) {
        match additional_platforms {
            Some(platforms) => {
                let mut indexes = self.indexes.write().await;

                match indexes.unspecified.get(&user_id) {
                    Some(unspecified_session) => {
                        unspecified_session.online_platforms.add(&platforms);
                    },
                    None => {
                        let unspecified_session =
                            UnspecifiedChannelSession::new(user_id);
                        unspecified_session.online_platforms.add(&platforms);

                        indexes
                            .unspecified
                            .insert(user_id, unspecified_session.into());
                    },
                };

                for platform in platforms {
                    match platform {
                        Platform::Bancho => {
                            if let Entry::Vacant(e) =
                                indexes.bancho.entry(user_id)
                            {
                                e.insert(BanchoChannelSession(user_id).into());
                                self.counter.platform_bancho.add(1);
                            }
                        },
                        Platform::Lazer => {
                            if let Entry::Vacant(e) =
                                indexes.lazer.entry(user_id)
                            {
                                e.insert(LazerChannelSession(user_id).into());
                                self.counter.platform_lazer.add(1);
                            }
                        },
                        Platform::Web => {
                            if let Entry::Vacant(e) = indexes.web.entry(user_id)
                            {
                                e.insert(WebChannelSession(user_id).into());
                                self.counter.platform_web.add(1);
                            }
                        },
                    }
                }
            },
            None => {
                let mut indexes = self.indexes.write().await;
                if let Entry::Vacant(e) = indexes.unspecified.entry(user_id) {
                    e.insert(UnspecifiedChannelSession::new(user_id).into());
                    self.counter.unspecified.add(1);
                }
            },
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
        platforms: Option<&[Platform]>,
    ) {
        match platforms {
            Some(platforms) => {
                let mut indexes = self.indexes.write().await;

                match indexes.unspecified.get(user_id) {
                    Some(unspecified_session) => {
                        unspecified_session.online_platforms.remove(platforms);
                    },
                    None => return,
                };

                for platform in platforms {
                    match platform {
                        Platform::Bancho => indexes
                            .bancho
                            .remove(user_id)
                            .map(|_| self.counter.platform_bancho.sub(1)),
                        Platform::Lazer => indexes
                            .lazer
                            .remove(user_id)
                            .map(|_| self.counter.platform_lazer.sub(1)),
                        Platform::Web => indexes
                            .web
                            .remove(user_id)
                            .map(|_| self.counter.platform_web.sub(1)),
                    };
                }
            },
            None => {
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
            },
        }
    }

    #[inline]
    pub async fn user_exists(
        &self,
        user_id: &i32,
        platforms: Option<&Platform>,
    ) -> bool {
        let indexes = self.indexes.read().await;

        match platforms {
            Some(platforms) => match platforms {
                Platform::Bancho => indexes.bancho.contains_key(user_id),
                Platform::Lazer => indexes.lazer.contains_key(user_id),
                Platform::Web => indexes.web.contains_key(user_id),
            },
            None => indexes.unspecified.contains_key(user_id),
        }
    }
}

#[derive(Debug, Default)]
pub struct ChannelMetadata {
    pub id: u64,
    pub name: Atomic<String>,
    pub channel_type: ChannelType,
    pub description: AtomicOption<String>,
}

#[derive(Debug, Default)]
pub struct Channel {
    pub metadata: ChannelMetadata,
    pub sessions: ChannelSessions,
    pub created_at: DateTime<Utc>,
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
            created_at: Utc::now(),
        }
    }

    #[inline]
    pub fn session_count(&self, platform: Option<&Platform>) -> u64 {
        self.sessions.counter.value(platform)
    }

    #[inline]
    pub fn rpc_channel_info(&self) -> ChannelInfo {
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
                unspecified: self.session_count(None),
                bancho: self.session_count(Some(&Platform::Bancho)),
                lazer: self.session_count(Some(&Platform::Lazer)),
                web: self.session_count(Some(&Platform::Web)),
            }),
            users: None,
        }
    }
}
