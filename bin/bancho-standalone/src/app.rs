use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application};
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_repositories::users::{DynUsersRepository, UsersRepositoryImpl};
use peace_services::{
    bancho::*,
    bancho_state::*,
    gateway::bancho_endpoints::{
        repository::{BanchoGatewayRepositoryImpl, DynBanchoGatewayRepository},
        routes::BanchoRouter,
        *,
    },
};
use std::sync::Arc;
use utoipa::OpenApi;

#[peace_config]
#[command(
    name = "bancho-standalone",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct GatewayConfig {
    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    #[command(flatten)]
    pub peace_db: PeaceDbConfig,
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
        let peace_db_conn = self
            .cfg
            .peace_db
            .connect()
            .await
            .expect("failed to connect peace db, please check.");

        let bancho_state_service = Arc::new(BanchoStateServiceImpl::Local(
            BanchoStateServiceLocal::default(),
        )) as DynBanchoStateService;

        let users_repository = Arc::new(UsersRepositoryImpl::new(peace_db_conn))
            as DynUsersRepository;

        let bancho_service =
            Arc::new(BanchoServiceImpl::Local(BanchoServiceLocal::new(
                users_repository,
                bancho_state_service.clone(),
            ))) as DynBanchoService;

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
        BanchoEndpointsDocs::openapi()
    }
}
