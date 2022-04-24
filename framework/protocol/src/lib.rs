#[macro_use(async_trait)]
extern crate async_trait;

use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub type SpaceInner = HashMap<&'static str, Server>;

pub type Readable<T> = RwLockReadGuard<'static, T>;
pub type Writeable<T> = RwLockWriteGuard<'static, T>;

pub type ReadableSpace = Readable<Space>;
pub type WriteableSpace = Writeable<Space>;

pub struct Server {
    pub sessions: SessionStorage,
    pub channels: ChatChannel,
}

pub struct ChatChannel {}

pub struct SessionStorage {}

pub struct Space {
    pub _inner: SpaceInner,
}

impl Space {
    pub fn new() -> Self {
        Self {
            _inner: HashMap::new(),
        }
    }
}

impl Deref for Space {
    type Target = SpaceInner;

    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

impl DerefMut for Space {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._inner
    }
}

pub enum Instance<T>
where
    T: 'static,
{
    Locked(&'static RwLock<T>),
    Writeable(Writeable<T>),
    Readable(Readable<T>),
}

pub struct SpaceStore;

impl SpaceStore {
    #[inline]
    pub fn get() -> &'static RwLock<Space> {
        static SPACE: Lazy<RwLock<Space>> = Lazy::new(|| RwLock::new(Space::new()));
        &SPACE
    }

    #[inline]
    pub async fn write() -> Writeable<Space> {
        Self::get().write().await
    }

    #[inline]
    pub async fn read() -> Readable<Space> {
        Self::get().read().await
    }
}

#[async_trait]
pub trait InstanceFactory<T> {
    fn get1() -> Instance<T>;
    async fn write1() -> Instance<T>;
    async fn read1() -> Instance<T>;
}

#[async_trait]
impl InstanceFactory<Space> for SpaceStore {
    fn get1() -> Instance<Space> {
        Instance::Locked(Self::get())
    }

    async fn write1() -> Instance<Space> {
        Instance::Writeable(Self::get().write().await)
    }

    async fn read1() -> Instance<Space> {
        Instance::Readable(Self::get().read().await)
    }
}

#[macro_export]
macro_rules! get_space {
    () => {
        SpaceStore::get()
    };
}

#[macro_export]
macro_rules! write_space {
    () => {
        SpaceStore::write().await
    };
}

#[macro_export]
macro_rules! read_space {
    () => {
        SpaceStore::read().await
    };
}

pub mod prelude {
    pub use crate::{get_space, read_space, write_space, SpaceStore};
}
