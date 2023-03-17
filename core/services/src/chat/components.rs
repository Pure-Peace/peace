use chrono::{DateTime, Utc};
use peace_pb::chat::ChannelQuery;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::RwLock;
use tools::atomic::{
    Atomic, AtomicOperation, AtomicOption, AtomicValue, Usize,
};

#[derive(Debug, Default, Clone)]
pub struct ChannelIndexes {
    pub channel_id: HashMap<u32, Arc<Channel>>,
    pub channel_name: HashMap<String, Arc<Channel>>,
}

impl Deref for ChannelIndexes {
    type Target = HashMap<u32, Arc<Channel>>;

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
    pub async fn create(&self, channel: Channel) -> Arc<Channel> {
        let channel = Arc::new(channel);

        let () = {
            let mut indexes = self.write().await;

            if let Some(old_channel) = self
                .get_inner(&mut indexes, &ChannelQuery::ChannelId(channel.id))
            {
                self.delete_inner(
                    &mut indexes,
                    &old_channel.id,
                    &old_channel.name.load(),
                );
            }

            indexes.channel_id.insert(channel.id.clone(), channel.clone());
            indexes
                .channel_name
                .insert(channel.name.to_string(), channel.clone());
        };

        self.len.add(1);

        channel
    }

    #[inline]
    pub async fn delete(&self, query: &ChannelQuery) -> Option<Arc<Channel>> {
        let mut indexes = self.write().await;

        let channel = self.get_inner(&indexes, query)?;

        self.delete_inner(&mut indexes, &channel.id, &channel.name.load())
    }

    #[inline]
    pub fn delete_inner(
        &self,
        indexes: &mut ChannelIndexes,
        channel_id: &u32,
        channel_name: &str,
    ) -> Option<Arc<Channel>> {
        let mut removed = None;

        indexes
            .channel_id
            .remove(channel_id)
            .and_then(|s| Some(removed = Some(s)));
        indexes
            .channel_name
            .remove(channel_name)
            .and_then(|s| Some(removed = Some(s)));

        if removed.is_some() {
            self.len.sub(1);
        }

        removed
    }

    #[inline]
    pub async fn get(&self, query: &ChannelQuery) -> Option<Arc<Channel>> {
        let indexes = self.read().await;
        self.get_inner(&indexes, query)
    }

    #[inline]
    pub fn get_inner(
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
    pub async fn exists(&self, query: &ChannelQuery) -> bool {
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
    pub async fn clear(&self) {
        let mut indexes = self.write().await;
        indexes.channel_id.clear();
        indexes.channel_name.clear();

        self.len.set(0);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len.val()
    }
}

#[derive(Debug, Default)]
pub struct Channel {
    pub id: u32,
    pub name: Atomic<String>,
    pub description: AtomicOption<String>,
    pub users: RwLock<Vec<i32>>,
    pub created_at: DateTime<Utc>,
}

impl Channel {
    pub fn new(
        id: u32,
        name: Atomic<String>,
        description: AtomicOption<String>,
        users: Vec<i32>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            users: users.into(),
            created_at: Utc::now(),
        }
    }
}
