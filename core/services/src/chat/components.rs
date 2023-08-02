use crate::bancho_state::{BanchoMessageData, BanchoMessageQueue};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use clap_serde_derive::ClapSerde;
use infra_packets::{Packet, PacketsQueue};
use infra_users::{
    BaseSession, BaseSessionData, CreateSessionDto, UserIndexes, UserStore,
};
use peace_domain::chat::{ChannelType, Platform};
use peace_pb::chat::ChannelQuery;
use peace_snapshot::{cli_snapshot_config, CreateSnapshot, SnapshotType};
use peace_unique_id::Ulid;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};
use tokio::sync::RwLock;
use tools::atomic::{
    Atomic, AtomicOperation, AtomicOption, AtomicValue, Usize, U32,
};

pub type SessionIndexes = UserIndexes<ChatSession>;
pub type UserSessions = UserStore<ChatSession>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChatSessionData {
    pub base: BaseSessionData,
    pub extends: ChatSessionExtendData,
}

#[derive(Debug, Default)]
pub struct ChatSession {
    pub base: BaseSession,
    pub extends: ChatSessionExtend,
}

#[async_trait]
impl CreateSnapshot<ChatSessionData> for ChatSession {
    async fn create_snapshot(&self) -> ChatSessionData {
        ChatSessionData {
            base: self.base.to_session_data(),
            extends: self.extends.create_snapshot().await,
        }
    }
}

impl From<ChatSessionData> for ChatSession {
    fn from(d: ChatSessionData) -> Self {
        Self { base: d.base.into(), extends: d.extends.into() }
    }
}

impl Deref for ChatSession {
    type Target = BaseSession;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ChatSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ChatSession {
    pub fn new(
        CreateSessionDto {
            user_id,
            username,
            username_unicode,
            privileges,
            extends,
        }: CreateSessionDto<ChatSessionExtend>,
    ) -> Self {
        Self {
            base: BaseSession::new(
                user_id,
                username,
                username_unicode,
                privileges,
            ),
            extends,
        }
    }
}

#[derive(Debug, Default)]
pub struct JoinedChannel {
    pub ptr: Atomic<Weak<Channel>>,
    pub message_index: Atomic<Ulid>,
    pub joined_time: DateTime<Utc>,
}

impl From<Weak<Channel>> for JoinedChannel {
    fn from(ptr: Weak<Channel>) -> Self {
        Self {
            ptr: ptr.into(),
            message_index: Default::default(),
            joined_time: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinedChannelData {
    pub channel_id: u64,
    pub message_index: Ulid,
    pub joined_time: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct BanchoChatExt {
    pub packets_queue: PacketsQueue,
    pub notify_index: Atomic<Ulid>,
}

impl From<BanchoChatExtData> for BanchoChatExt {
    fn from(data: BanchoChatExtData) -> Self {
        Self {
            packets_queue: PacketsQueue::from(data.packets_queue),
            notify_index: data.notify_index.into(),
        }
    }
}

impl From<PacketsQueue> for BanchoChatExt {
    fn from(packets_queue: PacketsQueue) -> Self {
        Self { packets_queue, ..Default::default() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanchoChatExtData {
    pub packets_queue: Vec<Packet>,
    pub notify_index: Ulid,
}

#[async_trait]
impl CreateSnapshot<BanchoChatExtData> for BanchoChatExt {
    async fn create_snapshot(&self) -> BanchoChatExtData {
        BanchoChatExtData {
            packets_queue: self.packets_queue.create_snapshot().await,
            notify_index: *self.notify_index.load().as_ref(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ChatSessionExtend {
    pub platforms: Atomic<Platform>,
    pub bancho_ext: AtomicOption<BanchoChatExt>,
    pub joined_channels: RwLock<HashMap<u64, Arc<JoinedChannel>>>,
    pub channel_count: U32,
}

impl From<ChatSessionExtendData> for ChatSessionExtend {
    fn from(data: ChatSessionExtendData) -> Self {
        let channel_count = U32::new(data.joined_channels.len() as u32);
        Self {
            platforms: Platform::from(data.platforms).into(),
            bancho_ext: data.bancho_ext.map(|d| d.into()).into(),
            joined_channels: RwLock::new(HashMap::from_iter(
                data.joined_channels.into_iter().map(|j| {
                    (
                        j.channel_id,
                        Arc::new(JoinedChannel {
                            ptr: Weak::new().into(),
                            message_index: j.message_index.into(),
                            joined_time: j.joined_time,
                        }),
                    )
                }),
            )),
            channel_count,
        }
    }
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

    pub async fn collect_joined_channels(&self) -> Vec<JoinedChannelData> {
        let mut channels =
            Vec::with_capacity(self.channel_count.val() as usize);

        for (channel_id, channel) in self.joined_channels.read().await.iter() {
            channels.push(JoinedChannelData {
                channel_id: *channel_id,
                message_index: *channel.message_index.load().as_ref(),
                joined_time: channel.joined_time,
            });
        }

        channels
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChatSessionExtendData {
    pub platforms: i32,
    pub bancho_ext: Option<BanchoChatExtData>,
    pub joined_channels: Vec<JoinedChannelData>,
}

#[async_trait]
impl CreateSnapshot<ChatSessionExtendData> for ChatSessionExtend {
    async fn create_snapshot(&self) -> ChatSessionExtendData {
        ChatSessionExtendData {
            platforms: self.platforms.load().bits(),
            bancho_ext: match self.bancho_ext.load().as_deref() {
                Some(ext) => Some(ext.create_snapshot().await),
                None => None,
            },
            joined_channels: self.collect_joined_channels().await,
        }
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

impl ChannelIndexes {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channel_id: HashMap::with_capacity(capacity),
            channel_name: HashMap::with_capacity(capacity),
            public_channels: HashMap::with_capacity(capacity),
        }
    }

    pub fn add_channel(&mut self, channel: Arc<Channel>) {
        self.channel_id.insert(channel.id, channel.clone());
        self.channel_name.insert(channel.name.to_string(), channel.clone());
        if channel.channel_type == ChannelType::Public {
            self.public_channels.insert(channel.id, channel);
        }
    }

    pub fn remove_channel(
        &mut self,
        channel_id: &u64,
        channel_name: &str,
    ) -> Option<Arc<Channel>> {
        let mut removed = None;

        if let Some(s) = self.channel_id.remove(channel_id) {
            removed = Some(s);
        }
        if let Some(s) = self.channel_name.remove(channel_name) {
            removed = Some(s);
        }
        if let Some(s) = self.public_channels.remove(channel_id) {
            removed = Some(s);
        }

        removed
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
    pub message_queue: Arc<BanchoMessageQueue>,
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
            message_queue: Arc::new(BanchoMessageQueue::default()),
            created_at: Utc::now(),
        }
    }

    pub async fn join(session: &Arc<ChatSession>, channel: &Arc<Channel>) {
        const LOG_TARGET: &str = "chat::channel::join";

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
                    ptr: Arc::downgrade(channel).into(),
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

        info!(
            target: LOG_TARGET,
            "User {}({}) joined into channel: {}({}) ",
            session.username.load(),
            session.user_id,
            channel.name.load(),
            channel.id
        );
    }

    pub async fn remove(session: &Arc<ChatSession>, channel: &Arc<Channel>) {
        const LOG_TARGET: &str = "chat::channel::remove";

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

        info!(
            target: LOG_TARGET,
            "User {}({}) leaved from channel: {}({}) ",
            session.username.load(),
            session.user_id,
            channel.name.load(),
            channel.id
        );
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub id: u64,
    pub name: String,
    pub channel_type: ChannelType,
    pub description: Option<String>,
    pub users: Vec<i32>,
    pub min_msg_index: Option<Ulid>,
    pub message_queue: Vec<BanchoMessageData>,
    pub created_at: DateTime<Utc>,
}

impl ChannelData {
    pub async fn from_channel(ch: &Channel) -> Self {
        ChannelData {
            id: ch.id,
            name: ch.name.to_string(),
            channel_type: ch.channel_type,
            description: ch
                .description
                .load()
                .as_deref()
                .map(|s| s.to_string()),
            users: ch.users.read().await.keys().copied().collect(),
            min_msg_index: ch.min_msg_index.load().as_deref().copied(),
            message_queue: ch
                .message_queue
                .read()
                .await
                .create_snapshot()
                .await,
            created_at: ch.created_at,
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
    pub fn from_indexes(indexes: ChannelIndexes) -> Self {
        let len = Usize::new(indexes.len());
        Self { indexes: RwLock::new(indexes), len }
    }

    #[inline]
    pub async fn create_channel(
        &self,
        channel: Channel,
        replace_if_exists: bool,
    ) -> Arc<Channel> {
        let channel = Arc::new(channel);
        let mut indexes = self.write().await;
        self.create_channel_inner(
            &mut indexes,
            channel.clone(),
            replace_if_exists,
        );

        channel
    }

    #[inline]
    pub fn create_channel_inner(
        &self,
        indexes: &mut ChannelIndexes,
        channel: Arc<Channel>,
        replace_if_exists: bool,
    ) {
        if let Some(old_channel) = self
            .get_channel_inner(indexes, &ChannelQuery::ChannelId(channel.id))
        {
            if !replace_if_exists {
                return;
            }

            self.remove_channel_inner(
                indexes,
                &old_channel.id,
                &old_channel.name.load(),
            );
        }

        indexes.add_channel(channel);

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
        let removed = indexes.remove_channel(channel_id, channel_name);

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

    pub async fn snapshot_channels(&self) -> Vec<ChannelData> {
        let mut channel_data = Vec::with_capacity(self.len.val());
        for channel in self.read().await.values() {
            channel_data
                .push(ChannelData::from_channel(channel.as_ref()).await);
        }

        channel_data
    }
}

cli_snapshot_config!(service: Chat);
