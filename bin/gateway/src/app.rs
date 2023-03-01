use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application, RpcClientConfig};
use peace_pb::{bancho_rpc, bancho_state_rpc};
use peace_services::{
    bancho::*,
    bancho_state::*,
    gateway::{
        bancho_endpoints::{
            repository::{
                BanchoGatewayRepositoryImpl, DynBanchoGatewayRepository,
            },
            routes::BanchoRouter,
            *,
        },
        docs::GatewayApiDocs,
    },
};
use std::sync::Arc;
use utoipa::OpenApi;

define_rpc_client_config!(
    service_name: bancho_rpc,
    config_name: BanchoRpcConfig
);

define_rpc_client_config!(
    service_name: bancho_state_rpc,
    config_name: BanchoStateRpcConfig,
    default_uri: "http://127.0.0.1:12345"
);

/// Command Line Interface (CLI) for Peace gateway service.
#[peace_config]
#[command(
    name = "peace-gateway",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct GatewayConfig {
    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    #[command(flatten)]
    pub bancho: BanchoRpcConfig,

    #[command(flatten)]
    pub bancho_state: BanchoStateRpcConfig,
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

        let bancho_state_service = Arc::new(BanchoStateServiceImpl::Remote(
            BanchoStateServiceRemote::new(bancho_state_rpc_client),
        )) as DynBanchoStateService;

        let bancho_service = Arc::new(BanchoServiceImpl::Remote(
            BanchoServiceRemote::new(bancho_rpc_client),
        )) as DynBanchoService;

        let bancho_gateway_repository =
            Arc::new(BanchoGatewayRepositoryImpl::new(
                bancho_service,
                bancho_state_service.clone(),
            )) as DynBanchoGatewayRepository;

        let bancho_gateway_service = Arc::new(BanchoGatewayServiceImpl::new(
            bancho_gateway_repository,
            bancho_state_service,
        )) as DynBanchoGatewayService;

        let bancho_router = BanchoRouter::new_router(bancho_gateway_service);

        bancho_router
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::openapi()
    }
}
