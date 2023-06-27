use super::{
    error::MessageError, DynMessageService, MessageService, PublishMessage,
};
use crate::{
    /* rpc_config::MessageRpcConfig, */ FromRpcClient, IntoService,
    RpcClient,
};
use async_nats::{
    jetstream::{self, context::PublishAckFuture, Context},
    Client,
};
use async_trait::async_trait;
use bytes::Bytes;
use derive_deref::Deref;
use peace_cfg::RpcClientConfig;
use std::{borrow::Cow, sync::Arc};
use tonic::transport::Channel;

/* pub struct MessageServiceBuilder;

impl MessageServiceBuilder {
    pub async fn build<I, R>(
        ed25519_pem_path: Option<&str>,
        signature_rpc_config: Option<&MessageRpcConfig>,
    ) -> DynMessageService
    where
        I: IntoService<DynMessageService> + From<SignerManager>,
        R: IntoService<DynMessageService>
            + FromRpcClient<Client = MessageRpcClient<Channel>>,
    {
        info!("initializing Message service...");
        let mut service = SignerManager::from_pem_file(
            ed25519_pem_path.unwrap_or(DEFAULT_ED25519_PEM_FILE_PATH),
        )
        .ok()
        .map(|signer| I::from(signer).into_service());

        if service.is_some() {
            info!("Message service init successful, type: \"Local\"");
            return service.unwrap();
        }

        if let Some(cfg) = signature_rpc_config {
            service = cfg
                .try_connect()
                .await
                .map(|client| {
                    info!("Message service init successful, type: \"Remote\"");
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
} */

#[derive(Clone)]
pub struct MessageServiceImpl {
    pub client: Client,
    pub stream: Context,
}

#[async_trait]
impl MessageService for MessageServiceImpl {}

impl IntoService<DynMessageService> for MessageServiceImpl {
    #[inline]
    fn into_service(self) -> DynMessageService {
        Arc::new(self) as DynMessageService
    }
}

impl From<Client> for MessageServiceImpl {
    #[inline]
    fn from(client: Client) -> Self {
        let stream = jetstream::new(client.clone());
        Self { client, stream }
    }
}

#[async_trait]
impl PublishMessage for MessageServiceImpl {
    #[inline]
    async fn publish<'a>(
        &self,
        subject: String,
        payload: Bytes,
    ) -> Result<(), MessageError> {
        Ok(self.client.publish(subject, payload).await?)
    }

    #[inline]
    async fn publish_stream<'a>(
        &self,
        subject: String,
        payload: Bytes,
    ) -> Result<PublishAckFuture, MessageError> {
        Ok(self
            .stream
            .publish(subject, payload)
            .await
            .map_err(MessageError::PublishStreamError)?)
    }
}

/* #[derive(Debug, Clone)]
pub struct MessageServiceRemote(MessageRpcClient<Channel>);

impl RpcClient for MessageServiceRemote {
    type Client = MessageRpcClient<Channel>;

    #[inline]
    fn client(&self) -> Self::Client {
        self.0.clone()
    }
}

impl FromRpcClient for MessageServiceRemote {
    fn from_client(client: Self::Client) -> Self {
        Self(client)
    }
}

impl MessageService for MessageServiceRemote {}

impl IntoService<DynMessageService> for MessageServiceRemote {
    #[inline]
    fn into_service(self) -> DynMessageService {
        Arc::new(self) as DynMessageService
    }
}

#[async_trait]
impl SignMessage for MessageServiceRemote {
    #[inline]
    async fn sign<'a>(
        &self,
        message: Cow<'a, str>,
    ) -> Result<String, MessageError> {
        self.client()
            .sign_message(SignMessageRequest { message: message.into_owned() })
            .await
            .map_err(MessageError::RpcError)
            .map(|resp| resp.into_inner().signature)
    }
}
 */
