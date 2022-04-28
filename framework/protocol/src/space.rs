use once_cell::sync::Lazy;
use std::{collections::HashMap, ops::Deref};

use crate::server::Server;

pub type SpaceInner = HashMap<&'static str, Server>;

pub struct Space;
crate::impl_locked_singleton!(SpaceInstance => Space);

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

    pub fn inner(&self) -> &SpaceInner {
        &self._inner
    }

    pub fn regist(&mut self, server: Server) -> Result<&mut Self, SpaceError> {
        if self.contains_key(server.name()) {
            return Err(SpaceError::RegistServerError("Server is already exists"));
        }
        self._inner.insert(server.name(), server);
        Ok(self)
    }
}

impl Deref for SpaceInstance {
    type Target = SpaceInner;

    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

pub mod prelude {
    pub use crate::{space::Space, traits::LockedSingleton};
}
