use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::atomic::{AtomicOption, AtomicValue, I64};

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
    pub expires: I64,
    pub last_update: I64,
}

impl<T> CachedAtomic<T> {
    #[inline]
    pub fn new(expires: I64) -> Self {
        Self {
            inner: AtomicOption::default(),
            expires,
            last_update: I64::default(),
        }
    }

    #[inline]
    pub fn new_with_init(expires: I64, init: T, last_update: I64) -> Self {
        Self { inner: AtomicOption::new(init), expires, last_update }
    }

    #[inline]
    pub fn update_time(&self) {
        self.last_update.set(Utc::now().timestamp().into());
    }

    #[inline]
    pub fn set_expries(&self, expires: i64) {
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
        Utc::now().timestamp() - self.expires.val() > self.last_update.val()
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
