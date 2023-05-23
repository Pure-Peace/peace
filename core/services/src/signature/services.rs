use super::{
    error::SignatureError, DynSignatureService, ReloadSigner, RequestPublicKey,
    SignMessage, SignatureService, VerifyMessage,
};
use crate::{
    rpc_config::SignatureRpcConfig, FromRpcClient, IntoService, RpcClient,
};
use async_trait::async_trait;
use derive_deref::Deref;
use ed25519::{self, Signature};
use peace_cfg::RpcClientConfig;
use peace_pb::signature::{
    signature_rpc_client::SignatureRpcClient, GetPublicKeyRequest,
    ReloadFromPemFileRequest, ReloadFromPemRequest, SignMessageRequest,
    VerifyMessageRequest,
};
use std::{borrow::Cow, sync::Arc};
use tonic::transport::Channel;
use tools::crypto::SignerManager;

const DEFAULT_ED25519_PEM_FILE_PATH: &str = "signature_svc_priv_key.pem";

pub struct SignatureServiceBuilder;

impl SignatureServiceBuilder {
    pub async fn build<I, R>(
        ed25519_pem_path: Option<&str>,
        signature_rpc_config: Option<&SignatureRpcConfig>,
    ) -> DynSignatureService
    where
        I: IntoService<DynSignatureService> + From<SignerManager>,
        R: IntoService<DynSignatureService>
            + FromRpcClient<Client = SignatureRpcClient<Channel>>,
    {
        info!("initializing Signature service...");
        let mut service = SignerManager::from_pem_file(
            ed25519_pem_path.unwrap_or(DEFAULT_ED25519_PEM_FILE_PATH),
        )
        .ok()
        .map(|signer| I::from(signer).into_service());

        if service.is_some() {
            info!("Signature service init successful, type: \"Local\"");
            return service.unwrap()
        }

        if let Some(cfg) = signature_rpc_config {
            service = cfg
                .connect_client()
                .await
                .map(|client| {
                    info!(
                        "Signature service init successful, type: \"Remote\""
                    );
                    R::from_client(client).into_service()
                })
                .ok();
        }

        service.unwrap_or_else(|| {
            info!("Generating new ed25519 private key...");
            let signer = SignerManager::new_rand();
            signer
                .store_to_pem_file(
                    ed25519_pem_path.unwrap_or(DEFAULT_ED25519_PEM_FILE_PATH),
                )
                .unwrap();
            I::from(signer).into_service()
        })
    }
}

#[derive(Clone, Deref)]
pub struct SignatureServiceImpl {
    pub signer: SignerManager,
}

#[async_trait]
impl SignatureService for SignatureServiceImpl {}

impl IntoService<DynSignatureService> for SignatureServiceImpl {
    #[inline]
    fn into_service(self) -> DynSignatureService {
        Arc::new(self) as DynSignatureService
    }
}

impl From<SignerManager> for SignatureServiceImpl {
    #[inline]
    fn from(signer: SignerManager) -> Self {
        Self { signer }
    }
}

#[async_trait]
impl SignMessage for SignatureServiceImpl {
    #[inline]
    async fn sign<'a>(
        &self,
        message: Cow<'a, str>,
    ) -> Result<String, SignatureError> {
        Ok(self.signer.sign(message.as_bytes())?.to_string())
    }
}

#[async_trait]
impl VerifyMessage for SignatureServiceImpl {
    #[inline]
    async fn verify<'a>(
        &self,
        message: Cow<'a, str>,
        signature_hex: Cow<'a, str>,
    ) -> Result<bool, SignatureError> {
        let signature = match Signature::from_slice(
            &hex::decode(signature_hex.as_bytes())
                .map_err(SignatureError::DecodeHexError)?,
        ) {
            Ok(sig) => sig,
            _ => return Ok(false),
        };
        Ok(self.signer.verify(message.as_bytes(), &signature).is_ok())
    }
}

#[async_trait]
impl ReloadSigner for SignatureServiceImpl {
    #[inline]
    async fn reload_from_pem<'a>(
        &self,
        ed25519_private_key: Cow<'a, str>,
    ) -> Result<(), SignatureError> {
        Ok(self.signer.reload_from_pem(ed25519_private_key.as_ref())?)
    }

    #[inline]
    async fn reload_from_pem_file<'a>(
        &self,
        ed25519_private_key_file_path: Cow<'a, str>,
    ) -> Result<(), SignatureError> {
        Ok(self
            .signer
            .reload_from_pem_file(ed25519_private_key_file_path.as_ref())?)
    }
}

#[async_trait]
impl RequestPublicKey for SignatureServiceImpl {
    #[inline]
    async fn public_key(&self) -> Result<String, SignatureError> {
        Ok(self.signer.public_key()?)
    }
}

#[derive(Debug, Clone)]
pub struct SignatureServiceRemote(SignatureRpcClient<Channel>);

impl RpcClient for SignatureServiceRemote {
    type Client = SignatureRpcClient<Channel>;

    #[inline]
    fn client(&self) -> Self::Client {
        self.0.clone()
    }
}

impl FromRpcClient for SignatureServiceRemote {
    fn from_client(client: Self::Client) -> Self {
        Self(client)
    }
}

impl SignatureService for SignatureServiceRemote {}

impl IntoService<DynSignatureService> for SignatureServiceRemote {
    #[inline]
    fn into_service(self) -> DynSignatureService {
        Arc::new(self) as DynSignatureService
    }
}

#[async_trait]
impl SignMessage for SignatureServiceRemote {
    #[inline]
    async fn sign<'a>(
        &self,
        message: Cow<'a, str>,
    ) -> Result<String, SignatureError> {
        self.client()
            .sign_message(SignMessageRequest { message: message.into_owned() })
            .await
            .map_err(SignatureError::RpcError)
            .map(|resp| resp.into_inner().signature)
    }
}

#[async_trait]
impl VerifyMessage for SignatureServiceRemote {
    #[inline]
    async fn verify<'a>(
        &self,
        message: Cow<'a, str>,
        signature: Cow<'a, str>,
    ) -> Result<bool, SignatureError> {
        self.client()
            .verify_message(VerifyMessageRequest {
                message: message.into_owned(),
                signature: signature.into_owned(),
            })
            .await
            .map_err(SignatureError::RpcError)
            .map(|resp| resp.into_inner().is_valid)
    }
}

#[async_trait]
impl ReloadSigner for SignatureServiceRemote {
    #[inline]
    async fn reload_from_pem<'a>(
        &self,
        ed25519_private_key: Cow<'a, str>,
    ) -> Result<(), SignatureError> {
        self.client()
            .reload_from_pem(ReloadFromPemRequest {
                ed25519_private_key: ed25519_private_key.into_owned(),
            })
            .await
            .map_err(SignatureError::RpcError)?;

        Ok(())
    }

    #[inline]
    async fn reload_from_pem_file<'a>(
        &self,
        ed25519_private_key_file_path: Cow<'a, str>,
    ) -> Result<(), SignatureError> {
        self.client()
            .reload_from_pem_file(ReloadFromPemFileRequest {
                ed25519_private_key_file_path: ed25519_private_key_file_path
                    .into_owned(),
            })
            .await
            .map_err(SignatureError::RpcError)?;

        Ok(())
    }
}

#[async_trait]
impl RequestPublicKey for SignatureServiceRemote {
    #[inline]
    async fn public_key(&self) -> Result<String, SignatureError> {
        self.client()
            .get_public_key(GetPublicKeyRequest {})
            .await
            .map_err(SignatureError::RpcError)
            .map(|resp| resp.into_inner().public_key)
    }
}
