use super::error::SignatureError;
use async_trait::async_trait;
use std::{borrow::Cow, sync::Arc};

pub type DynSignatureService = Arc<dyn SignatureService + Send + Sync>;

#[async_trait]
pub trait SignatureService {
    async fn sign<'a>(
        &self,
        message: Cow<'a, str>,
    ) -> Result<String, SignatureError>;

    async fn verify<'a>(
        &self,
        message: Cow<'a, str>,
        signature: Cow<'a, str>,
    ) -> Result<bool, SignatureError>;

    async fn reload_from_pem<'a>(
        &self,
        pem: Cow<'a, str>,
    ) -> Result<(), SignatureError>;

    async fn reload_from_pem_file<'a>(
        &self,
        path: Cow<'a, str>,
    ) -> Result<(), SignatureError>;

    async fn public_key(&self) -> Result<String, SignatureError>;
}
