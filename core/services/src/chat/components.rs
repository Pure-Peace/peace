use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tools::atomic::{Atomic, AtomicOption};

#[derive(Debug, Default, PartialEq)]
pub enum ChannelType {
    #[default]
    Private,
    Public,
    Group,
    Multiplayer,
    Spectaor,
}

#[derive(Debug, Default)]
pub struct Channel {
    pub id: u32,
    pub name: Atomic<String>,
    pub channel_type: ChannelType,
    pub description: AtomicOption<String>,
    pub users: RwLock<Vec<i32>>,
    pub created_at: DateTime<Utc>,
}

impl Channel {
    pub fn new(
        id: u32,
        name: Atomic<String>,
        channel_type: ChannelType,
        description: AtomicOption<String>,
        users: Vec<i32>,
    ) -> Self {
        Self {
            id,
            name,
            channel_type,
            description,
            users: users.into(),
            created_at: Utc::now(),
        }
    }
}
