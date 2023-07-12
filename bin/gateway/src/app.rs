use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, RpcClientConfig, WebApplication};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::{
    bancho::BanchoServiceRemote,
    bancho_state::BanchoStateServiceRemote,
    chat::ChatServiceRemote,
    gateway::{
        bancho_endpoints::{
            routes::{BanchoDebugRouter, BanchoRouter},
            BanchoHandlerServiceImpl, BanchoRoutingServiceImpl,
        },
        docs::GatewayApiDocs,
    },
    rpc_config::{
        BanchoRpcConfig, BanchoStateRpcConfig, ChatRpcConfig,
        SignatureRpcConfig,
    },
    signature::{
        SignatureServiceBuilder, SignatureServiceImpl, SignatureServiceRemote,
    },
    FromRpcClient, IntoService,
};
use std::sync::Arc;

/// PEACE Gateway service
#[peace_config]
#[command(name = "gateway", author, version, about, propagate_version = true)]
pub struct GatewayConfig {
    #[command(flatten)]
    pub runtime_cfg: RuntimeConfig,

    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    #[command(flatten)]
    pub bancho: BanchoRpcConfig,

    #[command(flatten)]
    pub bancho_state: BanchoStateRpcConfig,

    #[command(flatten)]
    pub chat: ChatRpcConfig,

    #[command(flatten)]
    pub signature_rpc_cfg: SignatureRpcConfig,

    #[arg(long)]
    pub ed25519_private_key_path: Option<String>,

    #[arg(long)]
    pub debug_endpoints: bool,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<GatewayConfig>,
}

impl App {
    pub fn new(cfg: Arc<GatewayConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl WebApplication for App {
    fn frame_cfg(&self) -> &ApiFrameConfig {
        &self.cfg.frame_cfg
    }

    async fn router<T: Clone + Sync + Send + 'static>(&self) -> Router<T> {
        let signature_service = SignatureServiceBuilder::build::<
            SignatureServiceImpl,
            SignatureServiceRemote,
        >(
            self.cfg.ed25519_private_key_path.as_deref(),
            Some(&self.cfg.signature_rpc_cfg),
        )
        .await;

        let bancho_rpc_client = self.cfg.bancho.connect().await;

        let bancho_state_rpc_client = self.cfg.bancho_state.connect().await;

        let chat_rpc_client = self.cfg.chat.connect().await;

        let bancho_state_service =
            BanchoStateServiceRemote::from_client(bancho_state_rpc_client)
                .into_service();

        let bancho_service =
            BanchoServiceRemote::from_client(bancho_rpc_client).into_service();

        let chat_service =
            ChatServiceRemote::from_client(chat_rpc_client).into_service();

        let bancho_handler_service = BanchoHandlerServiceImpl::new(
            bancho_service,
            bancho_state_service.clone(),
            chat_service,
            signature_service,
        )
        .into_service();

        let bancho_routing_service =
            BanchoRoutingServiceImpl::new(bancho_handler_service)
                .into_service();

        let mut router = BanchoRouter::new_router(bancho_routing_service);

        if self.cfg.debug_endpoints {
            router = router
                .merge(BanchoDebugRouter::new_router(bancho_state_service))
        }

        router
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::new_docs(self.cfg.debug_endpoints)
    }
}
