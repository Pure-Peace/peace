use super::traits::*;
use crate::chat::{
    Channel, ChannelMetadata, ChannelService, ChannelSessions, ChannelType,
    DynChannelService, Platform,
};
use async_trait::async_trait;
use peace_db::DatabaseConnection;
use peace_pb::chat::ChannelQuery;
use std::{collections::HashMap, ops::Deref, sync::Arc};
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
        self.channels.clear_all_channels().await
    }
}

impl ChannelCount for ChannelServiceImpl {
    #[inline]
    fn channel_count(&self) -> usize {
        self.channels.channel_count()
    }
}
