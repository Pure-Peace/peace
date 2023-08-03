use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use core_bancho::{BanchoRpcConfig, BanchoServiceRemote, DynBanchoService};
use core_bancho_state::{
    BanchoStateRpcConfig, BanchoStateServiceRemote, DynBanchoStateService,
};
use core_chat::{ChatRpcConfig, ChatServiceRemote};
use core_gateway::{
    bancho_endpoints::{
        routes::{BanchoDebugRouter, BanchoRouter},
        BanchoHandlerServiceImpl, BanchoRoutingServiceImpl,
        DynBanchoHandlerService, DynBanchoRoutingService,
    },
    docs::GatewayApiDocs,
};
use infra_services::{FromRpcClient, IntoService};
use peace_api::{ApiFrameConfig, RpcClientConfig, WebApplication};
use peace_pb::{
    bancho::bancho_rpc_client::BanchoRpcClient,
    bancho_state::bancho_state_rpc_client::BanchoStateRpcClient,
    chat::chat_rpc_client::ChatRpcClient,
};
use peace_runtime::cfg::RuntimeConfig;
use std::{net::SocketAddr, sync::Arc};
use tonic::transport::Channel;

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

    #[arg(long)]
    pub debug_endpoints: bool,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<GatewayConfig>,
    pub bancho_rpc_client: BanchoRpcClient<Channel>,
    pub bancho_state_rpc_client: BanchoStateRpcClient<Channel>,
    pub chat_rpc_client: ChatRpcClient<Channel>,
    pub bancho_state_service: DynBanchoStateService,
    pub bancho_service: DynBanchoService,
    pub bancho_handler_service: DynBanchoHandlerService,
    pub bancho_routing_service: DynBanchoRoutingService,
}

impl App {
    pub async fn initialize(cfg: Arc<GatewayConfig>) -> Self {
        let bancho_rpc_client = cfg.bancho.connect().await;

        let bancho_state_rpc_client = cfg.bancho_state.connect().await;

        let chat_rpc_client = cfg.chat.connect().await;

        let bancho_state_service = BanchoStateServiceRemote::from_client(
            bancho_state_rpc_client.clone(),
        )
        .into_service();

        let bancho_service =
            BanchoServiceRemote::from_client(bancho_rpc_client.clone())
                .into_service();

        let chat_service =
            ChatServiceRemote::from_client(chat_rpc_client.clone())
                .into_service();

        let bancho_handler_service = BanchoHandlerServiceImpl::new(
            bancho_service.clone(),
            bancho_state_service.clone(),
            chat_service.clone(),
        )
        .into_service();

        let bancho_routing_service =
            BanchoRoutingServiceImpl::new(bancho_handler_service.clone())
                .into_service();

        Self {
            cfg,
            bancho_rpc_client,
            bancho_state_rpc_client,
            chat_rpc_client,
            bancho_state_service,
            bancho_service,
            bancho_handler_service,
            bancho_routing_service,
        }
    }
}

#[async_trait]
impl WebApplication for App {
    fn frame_cfg(&self) -> &ApiFrameConfig {
        &self.cfg.frame_cfg
    }

    fn default_http_addr(&self) -> Option<SocketAddr> {
        Some("127.0.0.1:8000".parse().unwrap())
    }

    fn default_https_addr(&self) -> Option<SocketAddr> {
        Some("127.0.0.1:443".parse().unwrap())
    }

    async fn router<T: Clone + Sync + Send + 'static>(&self) -> Router<T> {
        let mut router =
            BanchoRouter::new_router(self.bancho_routing_service.clone());

        if self.cfg.debug_endpoints {
            router = router.merge(BanchoDebugRouter::new_router(
                self.bancho_state_service.clone(),
            ))
        }

        router
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::new_docs(self.cfg.debug_endpoints)
    }
}
