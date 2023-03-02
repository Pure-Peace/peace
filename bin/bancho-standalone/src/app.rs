use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application};
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_repositories::users::UsersRepositoryImpl;
use peace_services::{
    bancho::BanchoServiceImpl,
    bancho_state::{BanchoStateServiceImpl, UserSessionsServiceImpl},
    gateway::bancho_endpoints::{
        routes::{BanchoDebugRouter, BanchoRouter},
        BanchoDebugEndpointsDocs, BanchoEndpointsDocs,
        BanchoHandlerServiceImpl, BanchoRoutingServiceImpl,
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
pub struct BanchoStandaloneConfig {
    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    #[command(flatten)]
    pub peace_db: PeaceDbConfig,

    #[arg(long)]
    pub debug_endpoints: bool,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<BanchoStandaloneConfig>,
}

impl App {
    pub fn new(cfg: Arc<BanchoStandaloneConfig>) -> Self {
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

        let bancho_handler_service = BanchoHandlerServiceImpl::new(
            bancho_service,
            bancho_state_service.clone(),
        )
        .into_service();

        let bancho_routing_service = BanchoRoutingServiceImpl::new(
            bancho_handler_service,
            bancho_state_service.clone(),
        )
        .into_service();

        let mut router = BanchoRouter::new_router(bancho_routing_service);

        if self.cfg.debug_endpoints {
            router = router
                .merge(BanchoDebugRouter::new_router(bancho_state_service))
        }

        router
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        let mut docs = BanchoEndpointsDocs::openapi();

        if self.cfg.debug_endpoints {
            docs.merge(BanchoDebugEndpointsDocs::openapi())
        }

        docs
    }
}
