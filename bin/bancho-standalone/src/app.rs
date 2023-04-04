use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application};
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_pb::geoip;
use peace_repositories::users::UsersRepositoryImpl;
use peace_runtime::cfg::RuntimeConfig;
use peace_services::{
    bancho::{
        BanchoBackgroundServiceImpl, BanchoServiceImpl, PasswordServiceImpl,
    },
    bancho_state::{
        BanchoStateBackgroundServiceImpl, BanchoStateServiceImpl,
        UserSessionsServiceImpl,
    },
    chat::{ChannelServiceImpl, ChatServiceImpl},
    gateway::bancho_endpoints::{
        routes::{BanchoDebugRouter, BanchoRouter},
        BanchoDebugEndpointsDocs, BanchoEndpointsDocs,
        BanchoHandlerServiceImpl, BanchoRoutingServiceImpl,
    },
    geoip::GeoipServiceImpl,
};
use std::{path::PathBuf, sync::Arc};
use utoipa::OpenApi;

define_rpc_client_config!(
    service_name: geoip,
    config_name: GeoipRpcConfig,
    default_uri: "http://127.0.0.1:12346"
);

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
    pub runtime_cfg: RuntimeConfig,

    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    #[command(flatten)]
    pub peace_db: PeaceDbConfig,

    #[arg(long)]
    pub debug_endpoints: bool,

    #[command(flatten)]
    pub geoip: GeoipRpcConfig,

    #[arg(long, short = 'P')]
    pub geo_db_path: Option<PathBuf>,
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

        let user_session_service =
            UserSessionsServiceImpl::default().into_service();

        let bancho_state_background_service =
            BanchoStateBackgroundServiceImpl::new(user_session_service.clone())
                .into_service();

        bancho_state_background_service.start_all();

        let bancho_state_service = BanchoStateServiceImpl::local(
            user_session_service,
            bancho_state_background_service,
        )
        .into_service();

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn.clone()).into_service();

        let password_service = PasswordServiceImpl::default().into_service();

        let geoip_service = GeoipServiceImpl::local_or_remote(
            self.cfg.geo_db_path.as_ref().map(|path| {
                path.to_str().expect("failed to parse geo_db_path")
            }),
            Some(&self.cfg.geoip),
        )
        .await
        .into_service();

        let channel_service =
            ChannelServiceImpl::new(peace_db_conn).into_service();

        channel_service.initialize_public_channels().await;

        let chat_service = ChatServiceImpl::local(
            channel_service,
            bancho_state_service.clone(),
        )
        .into_service();

        let bancho_background_service =
            BanchoBackgroundServiceImpl::new(password_service.cache().clone())
                .into_service();

        bancho_background_service.start_all();

        let bancho_service = BanchoServiceImpl::local(
            users_repository,
            bancho_state_service.clone(),
            password_service,
            bancho_background_service,
            geoip_service,
            chat_service,
        )
        .into_service();

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
        let mut docs = BanchoEndpointsDocs::openapi();

        if self.cfg.debug_endpoints {
            docs.merge(BanchoDebugEndpointsDocs::openapi())
        }

        docs
    }
}
