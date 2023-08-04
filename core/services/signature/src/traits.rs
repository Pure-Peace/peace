use super::error::SignatureError;
use async_trait::async_trait;
use pb_base::ExecSuccess;
use std::{borrow::Cow, sync::Arc};

pub type DynSignatureService = Arc<dyn SignatureService + Send + Sync>;

pub trait SignatureService:
    SignMessage + VerifyMessage + ReloadSigner + RequestPublicKey
{
}

#[async_trait]
pub trait SignMessage {
    async fn sign<'a>(
        &self,
        message: Cow<'a, str>,
    ) -> Result<String, SignatureError>;
}

#[async_trait]
pub trait VerifyMessage {
    async fn verify<'a>(
        &self,
        message: Cow<'a, str>,
        signature: Cow<'a, str>,
    ) -> Result<bool, SignatureError>;
}

#[async_trait]
pub trait ReloadSigner {
    async fn reload_from_pem<'a>(
        &self,
        pem: Cow<'a, str>,
    ) -> Result<ExecSuccess, SignatureError>;

    async fn reload_from_pem_file<'a>(
        &self,
        path: Cow<'a, str>,
    ) -> Result<ExecSuccess, SignatureError>;
}

#[async_trait]
pub trait RequestPublicKey {
    async fn public_key(&self) -> Result<String, SignatureError>;
}
