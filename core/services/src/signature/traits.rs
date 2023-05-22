use super::error::SignatureError;
use async_trait::async_trait;
use peace_pb::signature::signature_rpc_client::SignatureRpcClient;
use std::{borrow::Cow, sync::Arc};
use tonic::transport::Channel;
use tools::crypto::SignerManager;

pub type DynSignatureService = Arc<dyn SignatureService + Send + Sync>;

pub trait SignatureService:
    SignMessage + VerifyMessage + ReloadSigner + RequestPublicKey
{
}

pub trait IntoSignatureService:
    SignatureService + Sized + Sync + Send + 'static
{
    fn into_service(self) -> DynSignatureService {
        Arc::new(self) as DynSignatureService
    }
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
    ) -> Result<(), SignatureError>;

    async fn reload_from_pem_file<'a>(
        &self,
        path: Cow<'a, str>,
    ) -> Result<(), SignatureError>;
}

#[async_trait]
pub trait RequestPublicKey {
    async fn public_key(&self) -> Result<String, SignatureError>;
}

pub trait FromSigner {
    fn from_signer(signer: SignerManager) -> Self;
}

pub trait FromClient {
    fn from_client(client: SignatureRpcClient<Channel>) -> Self;
}

pub trait SignatureServiceRpc {
    fn client(&self) -> SignatureRpcClient<Channel>;
}
