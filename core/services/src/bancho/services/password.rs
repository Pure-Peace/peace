use crate::bancho::{DynPasswordService, PasswordService};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use peace_domain::users::{Password, PasswordError};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub type HashedPassword = String;
pub type RawPassword = String;
pub type PasswordCacheStore =
    Arc<Mutex<HashMap<HashedPassword, PasswordCache>>>;

pub struct PasswordCache {
    raw: RawPassword,
    last_hit: DateTime<Utc>,
}

impl PasswordCache {
    pub fn new(raw: RawPassword) -> Self {
        Self { raw, last_hit: Utc::now() }
    }

    pub fn last_hit(&self) -> &DateTime<Utc> {
        &self.last_hit
    }
}

#[derive(Clone, Default)]
pub struct PasswordServiceImpl {
    cache: PasswordCacheStore,
}

impl PasswordServiceImpl {
    pub fn into_service(self) -> DynPasswordService {
        Arc::new(self) as DynPasswordService
    }
}

#[async_trait]
impl PasswordService for PasswordServiceImpl {
    async fn verify_password(
        &self,
        hashed_password: &str,
        password: &str,
    ) -> Result<(), PasswordError> {
        if let Some(cached) = self.cache.lock().await.get_mut(hashed_password) {
            if cached.raw == password {
                cached.last_hit = Utc::now();
                return Ok(());
            } else {
                return Err(PasswordError::InvalidPassword);
            }
        }

        let () = Password::verify_password(hashed_password, password)?;
        self.cache.lock().await.insert(
            hashed_password.to_owned(),
            PasswordCache::new(password.to_owned()),
        );

        Ok(())
    }
}
