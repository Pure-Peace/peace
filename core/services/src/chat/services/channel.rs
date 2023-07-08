use super::traits::*;
use crate::chat::{
    Channel, ChannelMetadata, ChannelService, ChannelSessions, ChannelType,
    DynChannelService, Platform,
};
use async_trait::async_trait;
use peace_db::DatabaseConnection;
use peace_pb::chat::ChannelQuery;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};
use tokio::sync::RwLock;
use tools::atomic::{AtomicOperation, AtomicValue, Usize};

#[derive(Debug, Default, Clone)]
pub struct ChannelIndexes {
    pub channel_id: HashMap<u64, Arc<Channel>>,
    pub channel_name: HashMap<String, Arc<Channel>>,
    pub channel_public: HashMap<u64, Arc<Channel>>,
}

impl Deref for ChannelIndexes {
    type Target = HashMap<u64, Arc<Channel>>;

    fn deref(&self) -> &Self::Target {
        &self.channel_id
    }
}

#[derive(Debug, Default)]
pub struct UserChannelRecords {
    pub channel_platforms: RwLock<HashMap<u64, Platform>>,
    pub platform_channels: RwLock<HashMap<Platform, HashSet<u64>>>,
}

impl<const N: usize> From<[(u64, Platform); N]> for UserChannelRecords {
    fn from(arr: [(u64, Platform); N]) -> Self {
        let mut channel_platforms = HashMap::with_capacity(N);
        let mut platform_channels = HashMap::with_capacity(N);

        for (channel_id, platforms) in arr {
            channel_platforms.insert(channel_id, platforms);

            let channels = HashSet::from([channel_id]);

            for p in Platform::all_platforms() {
                if platforms.contains(p) {
                    platform_channels.insert(p, channels.clone());
                }
            }
        }

        Self {
            channel_platforms: RwLock::new(channel_platforms),
            platform_channels: RwLock::new(platform_channels),
        }
    }
}

#[derive(Debug, Default)]
pub struct UserChannels {
    pub indexes: RwLock<HashMap<i32, Arc<UserChannelRecords>>>,
}

impl Deref for UserChannels {
    type Target = RwLock<HashMap<i32, Arc<UserChannelRecords>>>;

    fn deref(&self) -> &Self::Target {
        &self.indexes
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
    pub async fn add_channel(&self, channel: Channel) -> Arc<Channel> {
        let channel = Arc::new(channel);
        let mut indexes = self.write().await;
        self.add_channel_inner(&mut indexes, channel.clone());

        channel
    }

    #[inline]
    pub fn add_channel_inner(
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
            indexes.channel_public.insert(channel.id, channel.clone());
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
        if let Some(s) = indexes.channel_public.remove(channel_id) {
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
        indexes.channel_public.clear();

        self.len.set(0);
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.len.val()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChannelServiceImpl {
    pub channels: Arc<Channels>,
    pub user_channels: Arc<UserChannels>,
    pub peace_db_conn: DatabaseConnection,
}

impl ChannelServiceImpl {
    #[inline]
    pub fn into_service(self) -> DynChannelService {
        Arc::new(self) as DynChannelService
    }

    #[inline]
    pub fn new(peace_db_conn: DatabaseConnection) -> Self {
        Self { peace_db_conn, ..Default::default() }
    }
}

#[async_trait]
impl ChannelService for ChannelServiceImpl {}

impl ChannelStore for ChannelServiceImpl {
    #[inline]
    fn channels(&self) -> &Arc<Channels> {
        &self.channels
    }
}

impl UserChannelIndex for ChannelServiceImpl {
    #[inline]
    fn user_channels(&self) -> &Arc<UserChannels> {
        &self.user_channels
    }
}

#[async_trait]
impl InitializePublicChannels for ChannelServiceImpl {
    async fn initialize_public_channels(&self) {
        const LOG_TARGET: &str = "chat::channel::initialize_public_channels";

        // todo: load public channels from database
        let public_channels = vec![
            Channel::new(
                ChannelMetadata {
                    id: 0,
                    name: "#osu".to_string().into(),
                    channel_type: ChannelType::Public,
                    description: Some("default channel".to_string()).into(),
                },
                None,
            ),
            Channel::new(
                ChannelMetadata {
                    id: 1,
                    name: "#peace".to_string().into(),
                    channel_type: ChannelType::Public,
                    description: Some("peace channel".to_string()).into(),
                },
                None,
            ),
        ];

        {
            let mut indexes = self.channels.write().await;
            for channel in public_channels {
                self.channels.add_channel_inner(&mut indexes, channel.into());
            }
        };

        info!(target: LOG_TARGET, "Public channels successfully initialized.",);
    }
}

#[async_trait]
impl AddChannel for ChannelServiceImpl {
    #[inline]
    async fn add_channel(
        &self,
        metadata: ChannelMetadata,
        users: Vec<i32>,
    ) -> Arc<Channel> {
        const LOG_TARGET: &str = "chat::channel::create_channel";

        let channel = self
            .channels
            .add_channel(Channel::new(
                metadata,
                Some(ChannelSessions::new(users)),
            ))
            .await;

        info!(
            target: LOG_TARGET,
            "Channel created: {} [{}] ({:?})",
            channel.name.load(),
            channel.id,
            channel.channel_type
        );

        channel
    }
}

#[async_trait]
impl AddUser for ChannelServiceImpl {
    #[inline]
    async fn add_user(
        &self,
        query: &ChannelQuery,
        user_id: i32,
        platforms: Platform,
    ) -> Option<Arc<Channel>> {
        let channel = self.channels.get_channel(query).await?;
        channel.sessions.add_user(user_id, platforms).await;

        let records =
            { self.user_channels.read().await.get(&user_id).cloned() };

        match records {
            Some(records) => {
                records
                    .channel_platforms
                    .write()
                    .await
                    .entry(channel.id)
                    .and_modify(|p| p.add(&platforms))
                    .or_insert(platforms);

                let mut platform_channels =
                    records.platform_channels.write().await;

                for p in Platform::all_platforms() {
                    if platforms.contains(p) {
                        platform_channels
                            .entry(p)
                            .and_modify(|c| {
                                c.insert(channel.id);
                            })
                            .or_insert_with(|| HashSet::from([channel.id]));
                    }
                }
            },
            None => {
                let records = Arc::new(UserChannelRecords::from([(
                    channel.id, platforms,
                )]));

                for p in Platform::all_platforms() {
                    records
                        .platform_channels
                        .write()
                        .await
                        .entry(p)
                        .and_modify(|c| {
                            c.insert(channel.id);
                        })
                        .or_insert_with(|| HashSet::from([channel.id]));
                }

                self.user_channels.write().await.insert(user_id, records);
            },
        };

        Some(channel)
    }
}

#[async_trait]
impl RemoveUserPlatforms for ChannelServiceImpl {
    #[inline]
    async fn remove_user_platforms(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
        platforms: Platform,
    ) -> Option<Arc<Channel>> {
        let channel = self.channels.get_channel(query).await?;
        channel.sessions.remove_user_platforms(user_id, platforms).await;

        let records = { self.user_channels.read().await.get(user_id).cloned() };

        if let Some(records) = records {
            records
                .channel_platforms
                .write()
                .await
                .entry(channel.id)
                .and_modify(|p| p.remove(&platforms));

            let mut platform_channels = records.platform_channels.write().await;

            for p in Platform::all_platforms() {
                if platforms.contains(p) {
                    platform_channels.entry(p).and_modify(|c| {
                        c.remove(&channel.id);
                    });
                }
            }
        };

        Some(channel)
    }
}

#[async_trait]
impl RemoveUser for ChannelServiceImpl {
    #[inline]
    async fn remove_user(
        &self,
        query: &ChannelQuery,
        user_id: &i32,
    ) -> Option<Arc<Channel>> {
        let channel = self.channels.get_channel(query).await?;
        channel.sessions.remove_user(user_id).await;

        let records = { self.user_channels.read().await.get(user_id).cloned() };

        if let Some(records) = records {
            records.channel_platforms.write().await.remove(&channel.id);

            let mut platform_channels = records.platform_channels.write().await;

            for p in Platform::all_platforms() {
                platform_channels.entry(p).and_modify(|c| {
                    c.remove(&channel.id);
                });
            }
        };

        Some(channel)
    }
}

#[async_trait]
impl RemoveChannel for ChannelServiceImpl {
    #[inline]
    async fn remove_channel(
        &self,
        query: &ChannelQuery,
    ) -> Option<Arc<Channel>> {
        const LOG_TARGET: &str = "chat::channel::remove_channel";

        let channel = self.channels.remove_channel(query).await?;

        let all_user_records = {
            self.user_channels
                .read()
                .await
                .values()
                .cloned()
                .collect::<Vec<_>>()
        };

        for records in all_user_records {
            records.channel_platforms.write().await.remove(&channel.id);

            let mut platform_channels = records.platform_channels.write().await;

            for p in Platform::all_platforms() {
                platform_channels.entry(p).and_modify(|c| {
                    c.remove(&channel.id);
                });
            }
        }

        info!(
            target: LOG_TARGET,
            "Channel removed: {} [{}] ({:?})",
            channel.name.load(),
            channel.id,
            channel.channel_type
        );

        Some(channel)
    }
}

#[async_trait]
impl GetChannel for ChannelServiceImpl {
    #[inline]
    async fn get_channel(&self, query: &ChannelQuery) -> Option<Arc<Channel>> {
        self.channels.get_channel(query).await
    }
}

#[async_trait]
impl IsChannelExists for ChannelServiceImpl {
    #[inline]
    async fn is_channel_exists(&self, query: &ChannelQuery) -> bool {
        self.channels.is_channel_exists(query).await
    }
}

#[async_trait]
impl ClearAllChannels for ChannelServiceImpl {
    #[inline]
    async fn clear_all_channels(&self) {
        self.channels.clear_all_channels().await;
        self.user_channels.write().await.clear();
    }
}

impl ChannelCount for ChannelServiceImpl {
    #[inline]
    fn channel_count(&self) -> usize {
        self.channels.channel_count()
    }
}
