use crate::{
    bancho_state::{BanchoMessageQueue, BanchoPacketsQueue},
    users::{Session, UserIndexes, UserStore},
};
use bitmask_enum::bitmask;
use chrono::{DateTime, Utc};
use peace_pb::chat::ChannelQuery;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};
use tools::{
    atomic::{Atomic, AtomicOperation, AtomicOption, AtomicValue, Usize, U32},
    Ulid,
};

pub type ChatSession = Session<ChatExtend>;
pub type SessionIndexes = UserIndexes<ChatSession>;
pub type UserSessions = UserStore<ChatSession>;

#[derive(Debug, Default)]
pub struct BanchoChatExt {
    pub packets_queue: BanchoPacketsQueue,
    pub notify_index: Atomic<Ulid>,
}

impl From<BanchoPacketsQueue> for BanchoChatExt {
    fn from(packets_queue: BanchoPacketsQueue) -> Self {
        Self { packets_queue, ..Default::default() }
    }
}

#[derive(Debug, Default)]
pub struct ChatExtend {
    pub platforms: Atomic<Platform>,
    pub bancho_ext: Option<BanchoChatExt>,
}

impl ChatExtend {
    pub fn new(platforms: Platform, bancho_ext: Option<BanchoChatExt>) -> Self {
        Self { platforms: platforms.into(), bancho_ext }
    }
}

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

impl Platform {
    #[inline]
    pub const fn all_platforms() -> [Self; 3] {
        [Self::Bancho, Self::Lazer, Self::Web]
    }

    #[inline]
    pub fn add(&mut self, platforms: &Platform) {
        self.bits |= platforms.bits()
    }

    #[inline]
    pub fn remove(&mut self, platforms: &Platform) {
        self.bits &= !platforms.bits()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChannelIndexes {
    pub channel_id: HashMap<u64, Arc<Channel>>,
    pub channel_name: HashMap<String, Arc<Channel>>,
    pub public_channels: HashMap<u64, Arc<Channel>>,
}

impl Deref for ChannelIndexes {
    type Target = HashMap<u64, Arc<Channel>>;

    fn deref(&self) -> &Self::Target {
        &self.channel_id
    }
}

/* #[derive(Debug, Default, Clone)]
pub struct ChannelSessionIndexes {
    pub unspecified: HashMap<i32, Arc<UnspecifiedChannelSession>>,
    pub bancho: HashMap<i32, Arc<BanchoChannelSession>>,
    pub lazer: HashMap<i32, Arc<LazerChannelSession>>,
    pub web: HashMap<i32, Arc<WebChannelSession>>,
}

#[derive(Debug, Default)]
pub struct ChannelMetadata {
    pub id: u64,
    pub name: Atomic<String>,
    pub channel_type: ChannelType,
    pub description: AtomicOption<String>,
}
 */

#[derive(Debug, Default)]
pub struct Channel {
    pub id: u64,
    pub name: Atomic<String>,
    pub channel_type: ChannelType,
    pub description: AtomicOption<String>,

    pub users: Arc<RwLock<HashSet<i32>>>,
    pub user_count: U32,

    pub message_queue: Arc<Mutex<BanchoMessageQueue>>,
    pub created_at: DateTime<Utc>,
}

impl Channel {
    #[inline]
    pub fn new(
        id: u64,
        name: String,
        channel_type: ChannelType,
        description: Option<String>,
        users: Option<HashSet<i32>>,
    ) -> Self {
        let (user_count, users) = match users {
            Some(users) => (users.len() as u32, users),
            None => (0, HashSet::new()),
        };

        Self {
            id,
            name: name.into(),
            channel_type,
            description: description.into(),
            users: Arc::new(users.into()),
            user_count: user_count.into(),
            message_queue: Arc::new(Mutex::new(BanchoMessageQueue::default())),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Channels {
    pub indexes: RwLock<ChannelIndexes>,
    pub len: Usize,
}

impl Deref for Channels {
    type Target = RwLock<ChannelIndexes>;

    fn deref(&self) -> &Self::Target {
        &self.indexes
    }
}

impl Channels {
    #[inline]
    pub async fn create_channel(&self, channel: Channel) -> Arc<Channel> {
        let channel = Arc::new(channel);
        let mut indexes = self.write().await;
        self.create_channel_inner(&mut indexes, channel.clone());

        channel
    }

    #[inline]
    pub fn create_channel_inner(
        &self,
        indexes: &mut ChannelIndexes,
        channel: Arc<Channel>,
    ) {
        if let Some(old_channel) = self
            .get_channel_inner(indexes, &ChannelQuery::ChannelId(channel.id))
        {
            self.remove_channel_inner(
                indexes,
                &old_channel.id,
                &old_channel.name.load(),
            );
        }

        indexes.channel_id.insert(channel.id, channel.clone());
        indexes.channel_name.insert(channel.name.to_string(), channel.clone());
        if channel.channel_type == ChannelType::Public {
            indexes.public_channels.insert(channel.id, channel.clone());
        }

        self.len.add(1);
    }

    #[inline]
    pub async fn remove_channel(
        &self,
        query: &ChannelQuery,
    ) -> Option<Arc<Channel>> {
        let mut indexes = self.write().await;

        let channel = self.get_channel_inner(&indexes, query)?;

        self.remove_channel_inner(
            &mut indexes,
            &channel.id,
            &channel.name.load(),
        )
    }

    #[inline]
    pub fn remove_channel_inner(
        &self,
        indexes: &mut ChannelIndexes,
        channel_id: &u64,
        channel_name: &str,
    ) -> Option<Arc<Channel>> {
        let mut removed = None;

        if let Some(s) = indexes.channel_id.remove(channel_id) {
            removed = Some(s);
        }
        if let Some(s) = indexes.channel_name.remove(channel_name) {
            removed = Some(s);
        }
        if let Some(s) = indexes.public_channels.remove(channel_id) {
            removed = Some(s);
        }

        if removed.is_some() {
            self.len.sub(1);
        }

        removed
    }

    #[inline]
    pub async fn get_channel(
        &self,
        query: &ChannelQuery,
    ) -> Option<Arc<Channel>> {
        let indexes = self.read().await;
        self.get_channel_inner(&indexes, query)
    }

    #[inline]
    pub fn get_channel_inner(
        &self,
        indexes: &ChannelIndexes,
        query: &ChannelQuery,
    ) -> Option<Arc<Channel>> {
        match query {
            ChannelQuery::ChannelId(channel_id) => {
                indexes.channel_id.get(channel_id)
            },
            ChannelQuery::ChannelName(channel_name) => {
                indexes.channel_name.get(channel_name)
            },
        }
        .cloned()
    }

    #[inline]
    pub async fn is_channel_exists(&self, query: &ChannelQuery) -> bool {
        let indexes = self.read().await;
        match query {
            ChannelQuery::ChannelId(channel_id) => {
                indexes.channel_id.contains_key(channel_id)
            },
            ChannelQuery::ChannelName(channel_name) => {
                indexes.channel_name.contains_key(channel_name)
            },
        }
    }

    #[inline]
    pub async fn clear_all_channels(&self) {
        let mut indexes = self.write().await;
        indexes.channel_id.clear();
        indexes.channel_name.clear();
        indexes.public_channels.clear();

        self.len.set(0);
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.len.val()
    }
}
