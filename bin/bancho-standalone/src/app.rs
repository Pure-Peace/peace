use axum::{async_trait, Router};
use bancho_service::*;
use bancho_state_service::*;
use chat_service::*;
use clap_serde_derive::ClapSerde;
use gateway_service::bancho_endpoints::{routes::*, *};
use geoip_service::*;
use infra_services::IntoService;
use peace_api::{ApiFrameConfig, WebApplication};
use peace_db::{
    peace::{Peace, PeaceDbConfig},
    DbConfig, DbConnection,
};
use peace_repositories::users::{DynUsersRepository, UsersRepositoryImpl};
use peace_runtime::cfg::RuntimeConfig;
use signature_service::*;
use std::{net::SocketAddr, sync::Arc};
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

    #[command(flatten)]
    pub chat_snapshot: CliChatServiceSnapshotConfigs,

    #[command(flatten)]
    pub bancho_state_snapshot: CliBanchoStateServiceSnapshotConfigs,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<BanchoStandaloneConfig>,
    pub peace_db_conn: DbConnection<Peace>,
    pub user_sessions_service: DynUserSessionsService,
    pub signature_service: DynSignatureService,
    pub bancho_state_service: DynBanchoStateService,
    pub users_repository: DynUsersRepository,
    pub password_service: DynPasswordService,
    pub geoip_service: DynGeoipService,
    pub chat_service: DynChatService,
    pub chat_background_service: DynChatBackgroundService,
    pub chat_background_service_config: ChatBackgroundServiceConfigs,
    pub bancho_background_service: DynBanchoBackgroundService,
    pub bancho_background_service_config: BanchoBackgroundServiceConfigs,
    pub bancho_state_background_service: DynBanchoStateBackgroundService,
    pub bancho_state_background_service_config:
        BanchoStateBackgroundServiceConfigs,
    pub bancho_service: DynBanchoService,
    pub bancho_handler_service: DynBanchoHandlerService,
    pub bancho_routing_service: DynBanchoRoutingService,
}

impl App {
    pub async fn initialize(cfg: Arc<BanchoStandaloneConfig>) -> Self {
        let peace_db_conn = cfg
            .peace_db
            .connect()
            .await
            .expect("failed to connect peace db, please check.");

        let signature_service = SignatureServiceBuilder::build::<
            SignatureServiceImpl,
            SignatureServiceRemote,
        >(
            cfg.ed25519_private_key_path.as_deref(),
            Some(&cfg.signature_rpc_cfg),
        )
        .await;

        let bancho_state_service = BanchoStateServiceSnapshotLoader::load(
            &cfg.bancho_state_snapshot,
            signature_service.clone(),
        )
        .await;

        let user_sessions_service =
            bancho_state_service.user_sessions_service.clone();

        let bancho_state_service = bancho_state_service.into_service();

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn.clone()).into_service();

        let password_service = PasswordServiceImpl::default();
        let password_cache_store = password_service.cache_store().clone();
        let password_service = password_service.into_service();

        let geoip_service =
            GeoipServiceBuilder::build::<GeoipServiceImpl, GeoipServiceRemote>(
                cfg.geo_db_path.as_deref(),
                Some(&cfg.geoip),
            )
            .await;

        let chat_service = ChatServiceSnapshotLoader::load(
            &cfg.chat_snapshot,
            users_repository.clone(),
        )
        .await
        .into_service();

        let chat_background_service =
            Arc::new(ChatBackgroundServiceImpl::new(chat_service.clone()));

        let chat_background_service_config =
            ChatBackgroundServiceConfigs::with_cfg(
                &cfg.chat_background_service_configs,
            );

        chat_service
            .load_public_channels()
            .await
            .expect("Failed to load public channels");

        let bancho_background_service =
            BanchoBackgroundServiceImpl::new(password_cache_store)
                .into_service();

        let bancho_background_service_config = BanchoBackgroundServiceConfigs {
            password_caches_recycle: PasswordCachesRecycleConfig::buid_with_cfg(
                &cfg.bancho_background_service_configs,
            ),
        };

        let bancho_state_background_service =
            Arc::new(BanchoStateBackgroundServiceImpl::new(
                user_sessions_service.clone(),
            ));

        let bancho_state_background_service_config =
            BanchoStateBackgroundServiceConfigs::with_cfg(
                &cfg.bancho_state_background_service_configs,
            );

        chat_background_service
            .start_all(chat_background_service_config.clone());
        bancho_background_service
            .start_all(bancho_background_service_config.clone());
        bancho_state_background_service
            .start_all(bancho_state_background_service_config.clone());

        let bancho_service = BanchoServiceImpl::new(
            users_repository.clone(),
            bancho_state_service.clone(),
            password_service.clone(),
            bancho_background_service.clone(),
            geoip_service.clone(),
            chat_service.clone(),
        )
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
            peace_db_conn,
            user_sessions_service,
            signature_service,
            bancho_state_service,
            users_repository,
            password_service,
            geoip_service,
            chat_service,
            chat_background_service,
            chat_background_service_config,
            bancho_background_service,
            bancho_background_service_config,
            bancho_state_background_service,
            bancho_state_background_service_config,
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
        let mut docs = BanchoEndpointsDocs::openapi();

        if self.cfg.debug_endpoints {
            docs.merge(BanchoDebugEndpointsDocs::openapi())
        }

        docs
    }
}
