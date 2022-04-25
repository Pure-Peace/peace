use std::collections::HashMap;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::{
    server::Server,
    types::{Readable, Writeable},
};

pub type SpaceInner = HashMap<&'static str, Server>;

#[derive(Debug)]
pub enum SpaceError {
    RegistServerError(&'static str),
}

pub struct SpaceInstance {
    _inner: SpaceInner,
}

impl SpaceInstance {
    pub fn new() -> Self {
        Self {
            _inner: HashMap::new(),
        }
    }

    pub fn check_exists(&self, key: &'static str) -> bool {
        self._inner.contains_key(key)
    }

    pub fn regist(&mut self, server: Server) -> Result<&mut Self, SpaceError> {
        if self.check_exists(server.name()) {
            return Err(SpaceError::RegistServerError("Server is already exists"));
        }
        self._inner.insert(server.name(), server);
        Ok(self)
    }
}

pub struct Space;

impl Space {
    #[inline]
    pub fn get() -> &'static RwLock<SpaceInstance> {
        static SPACE: Lazy<RwLock<SpaceInstance>> = Lazy::new(|| RwLock::new(SpaceInstance::new()));
        &SPACE
    }

    #[inline]
    pub async fn write() -> Writeable<SpaceInstance> {
        Self::get().write().await
    }

    #[inline]
    pub async fn read() -> Readable<SpaceInstance> {
        Self::get().read().await
    }
}

pub mod prelude {
    pub use super::Space;
    pub use crate::{get_space, read_space, write_space};
}
