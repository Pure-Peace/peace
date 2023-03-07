use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application, RpcClientConfig};
use peace_pb::{bancho, bancho_state};
use peace_services::{
    bancho::BanchoServiceImpl,
    bancho_state::BanchoStateServiceImpl,
    gateway::{
        bancho_endpoints::{
            routes::{BanchoDebugRouter, BanchoRouter},
            BanchoHandlerServiceImpl, BanchoRoutingServiceImpl,
        },
        docs::GatewayApiDocs,
    },
};
use std::sync::Arc;

define_rpc_client_config!(
    service_name: bancho,
    config_name: BanchoRpcConfig
);

define_rpc_client_config!(
    service_name: bancho_state,
    config_name: BanchoStateRpcConfig,
    default_uri: "http://127.0.0.1:12345"
);

#[peace_config]
#[command(name = "gateway", author, version, about, propagate_version = true)]
pub struct GatewayConfig {
    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    #[command(flatten)]
    pub bancho: BanchoRpcConfig,

    #[command(flatten)]
    pub bancho_state: BanchoStateRpcConfig,

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
impl Application for App {
    fn frame_cfg(&self) -> &ApiFrameConfig {
        &self.cfg.frame_cfg
    }

    async fn router<T: Clone + Sync + Send + 'static>(&self) -> Router<T> {
        let bancho_rpc_client = self.cfg.bancho.connect_client().await.unwrap_or_else(|err| {
                error!("Unable to connect to the {err} service, please make sure the service is started.");
                panic!("{}", err)
            });

        let bancho_state_rpc_client =
            self.cfg.bancho_state.connect_client().await.unwrap_or_else(
                    |err| {
                        error!("Unable to connect to the {err} service, please make sure the service is started.");
                        panic!("{}", err)
                    },
                );

        let bancho_state_service =
            BanchoStateServiceImpl::remote(bancho_state_rpc_client)
                .into_service();

        let bancho_service =
            BanchoServiceImpl::remote(bancho_rpc_client).into_service();

        let bancho_handler_service = BanchoHandlerServiceImpl::new(
            bancho_service,
            bancho_state_service.clone(),
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
