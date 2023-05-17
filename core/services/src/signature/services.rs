use super::{error::SignatureError, DynSignatureService, SignatureService};
use crate::rpc_config::SignatureRpcConfig;
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
    pub async fn build(
        ed25519_pem_path: Option<&str>,
        signature_rpc_config: Option<&SignatureRpcConfig>,
    ) -> DynSignatureService {
        info!("initializing Signature service...");
        let mut service = SignerManager::from_pem_file(
            ed25519_pem_path.unwrap_or(DEFAULT_ED25519_PEM_FILE_PATH),
        )
        .ok()
        .map(|signer| SignatureServiceImpl::new(signer).into_service());

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
                    SignatureServiceRemote::new(client).into_service()
                })
                .ok();
        }

        service.unwrap_or_else(|| {
            info!("Generating new ed25519 private key...");
            let manager = SignerManager::new_rand();
            manager
                .store_to_pem_file(
                    ed25519_pem_path.unwrap_or(DEFAULT_ED25519_PEM_FILE_PATH),
                )
                .unwrap();
            SignatureServiceImpl::new(manager).into_service()
        })
    }
}

#[derive(Clone, Deref)]
pub struct SignatureServiceImpl {
    pub signer: SignerManager,
}

impl SignatureServiceImpl {
    #[inline]
    pub fn new(signer: SignerManager) -> Self {
        Self { signer }
    }

    pub fn into_service(self) -> DynSignatureService {
        Arc::new(self) as DynSignatureService
    }
}

#[derive(Debug, Clone)]
pub struct SignatureServiceRemote(SignatureRpcClient<Channel>);

impl SignatureServiceRemote {
    pub fn new(signature_rpc_client: SignatureRpcClient<Channel>) -> Self {
        Self(signature_rpc_client)
    }

    pub fn client(&self) -> SignatureRpcClient<Channel> {
        self.0.clone()
    }

    pub fn into_service(self) -> DynSignatureService {
        Arc::new(self) as DynSignatureService
    }
}

#[async_trait]
impl SignatureService for SignatureServiceImpl {
    #[inline]
    async fn sign<'a>(
        &self,
        message: Cow<'a, str>,
    ) -> Result<String, SignatureError> {
        Ok(self.signer.sign(message.as_bytes())?.to_string())
    }

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

    #[inline]
    async fn public_key(&self) -> Result<String, SignatureError> {
        Ok(self.signer.public_key()?)
    }
}

#[async_trait]
impl SignatureService for SignatureServiceRemote {
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

    #[inline]
    async fn public_key(&self) -> Result<String, SignatureError> {
        self.client()
            .get_public_key(GetPublicKeyRequest {})
            .await
            .map_err(SignatureError::RpcError)
            .map(|resp| resp.into_inner().public_key)
    }
}
