use async_trait::async_trait;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    atomic::{AtomicOption, AtomicValue, U64},
    Timestamp,
};

#[async_trait]
pub trait CachedValue {
    type Context;
    type Output;

    async fn fetch(&self, context: &Self::Context) -> Self::Output;
    async fn fetch_new(&self, context: &Self::Context) -> Self::Output;
}

pub struct Cached<T> {
    pub cache: T,
    pub expired: bool,
}

pub struct CachedAtomic<T> {
    pub inner: AtomicOption<T>,
    pub expires: U64,
    pub last_update: U64,
}

impl<T> CachedAtomic<T> {
    #[inline]
    pub fn new(expires: U64) -> Self {
        Self {
            inner: AtomicOption::default(),
            expires,
            last_update: U64::default(),
        }
    }

    #[inline]
    pub fn new_with_init(expires: U64, init: T, last_update: U64) -> Self {
        Self { inner: AtomicOption::new(init), expires, last_update }
    }

    #[inline]
    pub fn update_time(&self) {
        self.last_update.set(Timestamp::now());
    }

    #[inline]
    pub fn set_expries(&self, expires: u64) {
        self.expires.set(expires)
    }

    #[inline]
    pub fn update<F>(&self, mut f: F)
    where
        F: FnMut(&AtomicOption<T>),
    {
        f(&self.inner);
        self.update_time();
    }

    #[inline]
    pub fn is_expired(&self) -> bool {
        Timestamp::now() - self.expires.val() > self.last_update.val()
    }

    #[inline]
    pub fn get(&self) -> Option<Cached<Arc<T>>> {
        Some(Cached { cache: self.inner.val()?, expired: self.is_expired() })
    }

    #[inline]
    pub fn set(&self, t: Option<Arc<T>>) -> Option<Arc<T>> {
        let old = self.inner.swap(t);
        self.update_time();
        old
    }
}

pub struct CachedRwLock<T> {
    pub inner: RwLock<T>,
    pub expires: U64,
    pub last_update: U64,
}

impl<T> CachedRwLock<T> {
    #[inline]
    pub fn new(inner: RwLock<T>, expires: U64) -> Self {
        Self { inner, expires, last_update: U64::default() }
    }

    #[inline]
    pub fn new_with_init(expires: U64, init: T, last_update: U64) -> Self {
        Self { inner: RwLock::new(init), expires, last_update }
    }

    #[inline]
    pub fn update_time(&self) {
        self.last_update.set(Timestamp::now());
    }

    #[inline]
    pub fn set_expries(&self, expires: u64) {
        self.expires.set(expires)
    }

    #[inline]
    pub async fn update<F>(&self, mut f: F)
    where
        F: FnMut(
            &RwLock<T>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    {
        f(&self.inner).await;
        self.update_time();
    }

    #[inline]
    pub fn is_expired(&self) -> bool {
        Timestamp::now() - self.expires.val() > self.last_update.val()
    }

    #[inline]
    pub fn get(&self) -> Option<Cached<&RwLock<T>>> {
        Some(Cached { cache: &self.inner, expired: self.is_expired() })
    }
}
