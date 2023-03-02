use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application};
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_repositories::users::UsersRepositoryImpl;
use peace_services::{
    bancho::BanchoServiceImpl,
    bancho_state::{BanchoStateServiceImpl, UserSessionsServiceImpl},
    gateway::bancho_endpoints::{
        repository::BanchoGatewayRepositoryImpl, routes::BanchoRouter,
        BanchoEndpointsDocs, BanchoGatewayServiceImpl,
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

        let user_sessions = UserSessionsServiceImpl::default().into_service();

        let bancho_state_service =
            BanchoStateServiceImpl::local(user_sessions).into_service();

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn).into_service();

        let bancho_service = BanchoServiceImpl::local(
            users_repository,
            bancho_state_service.clone(),
        )
        .into_service();

        let bancho_gateway_repository = BanchoGatewayRepositoryImpl::new(
            bancho_service,
            bancho_state_service.clone(),
        )
        .into_service();

        let bancho_gateway_service = BanchoGatewayServiceImpl::new(
            bancho_gateway_repository,
            bancho_state_service,
        )
        .into_service();

        let bancho_router = BanchoRouter::new_router(bancho_gateway_service);

        bancho_router
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        BanchoEndpointsDocs::openapi()
    }
}
