use chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};
use tokio::sync::RwLock;
use tools::atomic::{Atomic, AtomicOption};

pub type ChannelUserMap = HashMap<i32, ChannelSession>;

#[derive(Debug, Copy, Clone, Default, PartialEq, Primitive)]
pub enum ChannelType {
    #[default]
    Private = 0,
    Public = 1,
    Group = 2,
    Multiplayer = 3,
    Spectaor = 4,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Primitive, Hash)]
pub enum SessionPlatform {
    #[default]
    Bancho = 0,
    Lazer = 1,
    Web = 2,
}

pub struct SessionPlatforms(pub Vec<SessionPlatform>);

impl SessionPlatforms {
    #[inline]
    pub fn into_inner(self) -> Vec<SessionPlatform> {
        self.0
    }
}

impl From<Vec<i32>> for SessionPlatforms {
    fn from(platforms: Vec<i32>) -> Self {
        Self(
            platforms
                .into_iter()
                .map(|p| SessionPlatform::try_from(p))
                .filter(|result| {
                    if result.is_err() {
                        warn!("Unsupported SessionPlatform: {:?}", result)
                    }
                    true
                })
                .map(|p| p.unwrap())
                .collect(),
        )
    }
}

#[derive(Debug, Default)]
pub struct ChannelSession {
    pub user_id: i32,
    pub online_platforms: Atomic<HashSet<SessionPlatform>>,
}

impl ChannelSession {
    #[inline]
    pub fn new(
        user_id: i32,
        online_platforms: impl IntoIterator<Item = SessionPlatform>,
    ) -> Self {
        Self {
            user_id,
            online_platforms: Atomic::new(HashSet::from_iter(online_platforms)),
        }
    }

    #[inline]
    pub fn new_offline(user_id: i32) -> Self {
        Self { user_id, ..Default::default() }
    }

    #[inline]
    pub fn is_offline(&self) -> bool {
        self.online_platforms.load().is_empty()
    }

    #[inline]
    pub fn online_platforms(&self) -> Option<Vec<SessionPlatform>> {
        let online_platforms = self.online_platforms.load();
        if online_platforms.is_empty() {
            return None;
        }

        Some(online_platforms.iter().map(|p| p.to_owned()).collect())
    }
}

#[derive(Debug, Default)]
pub struct Channel {
    pub id: u64,
    pub name: Atomic<String>,
    pub channel_type: ChannelType,
    pub description: AtomicOption<String>,
    pub channel_sessions: RwLock<ChannelUserMap>,
    pub created_at: DateTime<Utc>,
}

impl Deref for Channel {
    type Target = RwLock<ChannelUserMap>;

    fn deref(&self) -> &Self::Target {
        &self.channel_sessions
    }
}

impl Channel {
    #[inline]
    pub fn new(
        id: u64,
        name: Atomic<String>,
        channel_type: ChannelType,
        description: AtomicOption<String>,
        channel_sessions: Option<&[i32]>,
    ) -> Self {
        Self {
            id,
            name,
            channel_type,
            description,
            channel_sessions: channel_sessions
                .map(|channel_sessions| {
                    HashMap::from_iter(channel_sessions.into_iter().map(
                        |user_id| {
                            (*user_id, ChannelSession::new_offline(*user_id))
                        },
                    ))
                    .into()
                })
                .unwrap_or_default(),
            created_at: Utc::now(),
        }
    }

    #[inline]
    pub async fn join(
        &self,
        user_id: i32,
        platforms: Vec<SessionPlatform>,
    ) -> usize {
        let mut lock = self.write().await;
        Self::join_inner(&mut lock, user_id, platforms)
    }

    #[inline]
    pub fn join_inner(
        channel_sessions: &mut ChannelUserMap,
        user_id: i32,
        platforms: Vec<SessionPlatform>,
    ) -> usize {
        if let Some(session) = channel_sessions.get(&user_id) {
            if !platforms.is_empty() {
                let mut old = session.online_platforms.load().as_ref().clone();
                old.extend(platforms);

                session.online_platforms.store(old.into());
            }
        } else {
            channel_sessions
                .insert(user_id, ChannelSession::new(user_id, platforms));
        }

        channel_sessions.len()
    }

    #[inline]
    pub async fn delete(&self, user_id: &i32) -> usize {
        let mut channel_sessions = self.write().await;
        Self::delete_inner(&mut channel_sessions, user_id)
    }

    #[inline]
    pub fn delete_inner(
        channel_sessions: &mut ChannelUserMap,
        user_id: &i32,
    ) -> usize {
        channel_sessions.remove(user_id);
        channel_sessions.len()
    }

    #[inline]
    pub async fn leave(
        &self,
        user_id: &i32,
        platforms: &[SessionPlatform],
    ) -> usize {
        let channel_sessions = self.read().await;
        Self::leave_inner(&channel_sessions, user_id, platforms)
    }

    #[inline]
    pub fn leave_inner(
        channel_sessions: &ChannelUserMap,
        user_id: &i32,
        platforms: &[SessionPlatform],
    ) -> usize {
        if let Some(session) = channel_sessions.get(user_id) {
            if platforms.is_empty() {
                session.online_platforms.store(Default::default());
            } else {
                let mut old = session.online_platforms.load().as_ref().clone();
                for p in platforms {
                    old.remove(&p);
                }

                session.online_platforms.store(old.into());
            }
        }

        channel_sessions.len()
    }

    #[inline]
    pub async fn exists(&self, user_id: &i32) -> bool {
        let lock = self.read().await;
        Self::exists_inner(&lock, user_id)
    }

    #[inline]
    pub fn exists_inner(
        channel_sessions: &ChannelUserMap,
        user_id: &i32,
    ) -> bool {
        channel_sessions.contains_key(user_id)
    }
}
