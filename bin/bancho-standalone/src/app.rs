use axum::{async_trait, Router};
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, WebApplication};
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_repositories::users::UsersRepositoryImpl;
use peace_runtime::cfg::RuntimeConfig;
use peace_services::{
    bancho::*,
    bancho_state::*,
    chat::*,
    gateway::bancho_endpoints::{routes::*, *},
    geoip::*,
    rpc_config::*,
    signature::*,
    IntoService,
};
use std::sync::Arc;
use utoipa::OpenApi;

/// PEACE Bancho standalone (web) service
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
    pub bancho_state_background_service_configs:
        CliBanchoStateBackgroundServiceConfigs,

    #[command(flatten)]
    pub bancho_background_service_configs: CliBanchoBackgroundServiceConfigs,

    #[command(flatten)]
    pub chat_background_service_configs: CliChatBackgroundServiceConfigs,

    #[command(flatten)]
    pub geoip: GeoipRpcConfig,

    #[arg(long, short = 'P')]
    pub geo_db_path: Option<String>,

    #[command(flatten)]
    pub signature_rpc_cfg: SignatureRpcConfig,

    #[arg(long)]
    pub ed25519_private_key_path: Option<String>,
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
impl WebApplication for App {
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

        let user_session_service = Arc::new(UserSessionsServiceImpl::default());

        let signature_service = SignatureServiceBuilder::build::<
            SignatureServiceImpl,
            SignatureServiceRemote,
        >(
            self.cfg.ed25519_private_key_path.as_deref(),
            Some(&self.cfg.signature_rpc_cfg),
        )
        .await;

        let bancho_state_service = BanchoStateServiceImpl::new(
            user_session_service.clone(),
            signature_service.clone(),
        )
        .into_service();

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn.clone()).into_service();

        let password_service = PasswordServiceImpl::default();
        let password_cache_store = password_service.cache_store().clone();

        let geoip_service =
            GeoipServiceBuilder::build::<GeoipServiceImpl, GeoipServiceRemote>(
                self.cfg.geo_db_path.as_deref(),
                Some(&self.cfg.geoip),
            )
            .await;

        let chat_service =
            Arc::new(ChatServiceImpl::new(users_repository.clone()));

        let chat_background_service =
            Arc::new(ChatBackgroundServiceImpl::new(chat_service.clone()));

        let chat_background_service_config =
            ChatBackgroundServiceConfigs::with_cfg(
                &self.cfg.chat_background_service_configs,
            );

        chat_service.load_public_channels().await.expect("debugging");

        let bancho_background_service =
            BanchoBackgroundServiceImpl::new(password_cache_store)
                .into_service();

        let bancho_background_service_config = BanchoBackgroundServiceConfigs {
            password_caches_recycle: PasswordCachesRecycleConfig::buid_with_cfg(
                &self.cfg.bancho_background_service_configs,
            ),
        };

        let bancho_state_background_service = Arc::new(
            BanchoStateBackgroundServiceImpl::new(user_session_service.clone()),
        );

        let bancho_state_background_service_config =
            BanchoStateBackgroundServiceConfigs::with_cfg(
                &self.cfg.bancho_state_background_service_configs,
            );

        chat_background_service.start_all(chat_background_service_config);
        bancho_background_service.start_all(bancho_background_service_config);
        bancho_state_background_service
            .start_all(bancho_state_background_service_config);

        let bancho_service = BanchoServiceImpl::new(
            users_repository,
            bancho_state_service.clone(),
            password_service.into_service(),
            bancho_background_service,
            geoip_service,
            chat_service.clone(),
        )
        .into_service();

        let bancho_handler_service = BanchoHandlerServiceImpl::new(
            bancho_service,
            bancho_state_service.clone(),
            chat_service,
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
