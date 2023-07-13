use crate::{
    bancho_state::{BanchoMessageQueue, BanchoPacketsQueue},
    users::{Session, UserIndexes, UserStore},
};
use bitmask_enum::bitmask;
use chrono::{DateTime, Utc};
use peace_pb::chat::ChannelQuery;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Weak},
};
use tokio::sync::RwLock;
use tools::{
    atomic::{Atomic, AtomicOperation, AtomicOption, AtomicValue, Usize, U32},
    Ulid,
};

pub type ChatSession = Session<ChatSessionExtend>;
pub type SessionIndexes = UserIndexes<ChatSession>;
pub type UserSessions = UserStore<ChatSession>;

#[derive(Debug, Default)]
pub struct JoinedChannel {
    pub ptr: Weak<Channel>,
    pub message_index: Atomic<Ulid>,
    pub joined_time: DateTime<Utc>,
}

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
pub struct ChatSessionExtend {
    pub platforms: Atomic<Platform>,
    pub bancho_ext: AtomicOption<BanchoChatExt>,
    pub joined_channels: RwLock<HashMap<u64, Arc<JoinedChannel>>>,
    pub channel_count: U32,
}

impl ChatSessionExtend {
    pub fn new(
        platforms: Platform,
        bancho_ext: Option<BanchoChatExt>,
        joined_channels: Option<HashMap<u64, Arc<JoinedChannel>>>,
    ) -> Self {
        let joined_channels = joined_channels.unwrap_or_default();
        let channel_count = joined_channels.len();

        Self {
            platforms: platforms.into(),
            bancho_ext: bancho_ext.into(),
            joined_channels: RwLock::new(joined_channels),
            channel_count: U32::from(channel_count as u32),
        }
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

#[derive(Debug, Default)]
pub struct Channel {
    pub id: u64,
    pub name: Atomic<String>,
    pub channel_type: ChannelType,
    pub description: AtomicOption<String>,

    pub users: Arc<RwLock<HashMap<i32, Option<Weak<ChatSession>>>>>,
    pub user_count: U32,

    pub min_msg_index: AtomicOption<Ulid>,
    pub message_queue: Arc<RwLock<BanchoMessageQueue>>,
    pub created_at: DateTime<Utc>,
}

impl Channel {
    #[inline]
    pub fn new(
        id: u64,
        name: String,
        channel_type: ChannelType,
        description: Option<String>,
        users: Option<Vec<i32>>,
    ) -> Self {
        let (user_count, users) = match users {
            Some(users) => (
                users.len() as u32,
                users.into_iter().map(|user_id| (user_id, None)).collect(),
            ),
            None => (0, HashMap::new()),
        };

        Self {
            id,
            name: name.into(),
            channel_type,
            description: description.into(),
            users: Arc::new(users.into()),
            user_count: user_count.into(),
            min_msg_index: None.into(),
            message_queue: Arc::new(RwLock::new(BanchoMessageQueue::default())),
            created_at: Utc::now(),
        }
    }

    pub async fn join(
        session: &Arc<Session<ChatSessionExtend>>,
        channel: &Arc<Channel>,
    ) {
        channel.users.write().await.entry(session.user_id).or_insert_with(
            || {
                channel.user_count.add(1);
                Some(Arc::downgrade(session))
            },
        );

        session
            .extends
            .joined_channels
            .write()
            .await
            .entry(channel.id)
            .or_insert_with(|| {
                session.extends.channel_count.add(1);
                JoinedChannel {
                    ptr: Arc::downgrade(channel),
                    message_index: Ulid::default().into(),
                    joined_time: Utc::now(),
                }
                .into()
            });

        // notify to user's bancho client if possible
        if let Some(bancho_ext) = session.extends.bancho_ext.load().as_ref() {
            bancho_ext
                .packets_queue
                .push_packet(channel.join_packets().into())
                .await;
        }
    }

    pub async fn remove(
        session: &Arc<Session<ChatSessionExtend>>,
        channel: &Arc<Channel>,
    ) {
        if channel.users.write().await.remove(&session.user_id).is_some() {
            channel.user_count.sub(1);
        }

        if session
            .extends
            .joined_channels
            .write()
            .await
            .remove(&channel.id)
            .is_some()
        {
            session.extends.channel_count.sub(1);
        }

        // notify to user's bancho client if possible
        if let Some(bancho_ext) = session.extends.bancho_ext.load().as_ref() {
            bancho_ext
                .packets_queue
                .push_packet(channel.kick_packets().into())
                .await;
        }
    }

    #[inline]
    pub fn info_packets(&self) -> Vec<u8> {
        bancho_packets::server::ChannelInfo::pack(
            self.name.load().as_ref().into(),
            self.description
                .load()
                .as_deref()
                .map(|s| s.to_owned())
                .unwrap_or_default()
                .into(),
            self.user_count.val() as i16,
        )
    }

    #[inline]
    pub fn join_packets(&self) -> Vec<u8> {
        bancho_packets::server::ChannelJoin::pack(
            self.name.load().as_ref().into(),
        )
    }

    #[inline]
    pub fn kick_packets(&self) -> Vec<u8> {
        bancho_packets::server::ChannelKick::pack(
            self.name.load().as_ref().into(),
        )
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
