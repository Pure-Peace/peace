use crate::BanchoRpcImpl;
use bancho_service::*;
use bancho_state_service::{
    BanchoStateRpcConfig, BanchoStateServiceRemote, DynBanchoStateService,
};
use chat_service::{ChatRpcConfig, ChatServiceRemote, DynChatService};
use clap_serde_derive::ClapSerde;
use geoip_service::{
    DynGeoipService, GeoipRpcConfig, GeoipServiceBuilder, GeoipServiceImpl,
    GeoipServiceRemote,
};
use infra_services::{FromRpcClient, IntoService};
use peace_db::{
    peace::{Peace, PeaceDbConfig},
    DbConfig, DbConnection,
};
use peace_pb::{
    bancho::{bancho_rpc_server::BanchoRpcServer, BANCHO_DESCRIPTOR_SET},
    bancho_state::bancho_state_rpc_client::BanchoStateRpcClient,
    chat::chat_rpc_client::ChatRpcClient,
};
use peace_repositories::users::{DynUsersRepository, UsersRepositoryImpl};
use peace_rpc::{
    interceptor::client_ip, RpcApplication, RpcClientConfig, RpcFrameConfig,
};
use peace_runtime::cfg::RuntimeConfig;
use std::{net::SocketAddr, sync::Arc};
use tonic::{
    async_trait,
    transport::{server::Router, Channel, Server},
};

/// PEACE Bancho gRPC service
#[peace_config]
#[command(name = "bancho", author, version, about, propagate_version = true)]
pub struct BanchoConfig {
    #[command(flatten)]
    pub runtime_cfg: RuntimeConfig,

    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,

    #[command(flatten)]
    pub peace_db: PeaceDbConfig,

    #[command(flatten)]
    pub bancho_state: BanchoStateRpcConfig,

    #[command(flatten)]
    pub geoip: GeoipRpcConfig,

    #[command(flatten)]
    pub chat: ChatRpcConfig,

    #[command(flatten)]
    pub bancho_background_service_configs: CliBanchoBackgroundServiceConfigs,

    #[arg(long, short = 'P')]
    pub geo_db_path: Option<String>,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<BanchoConfig>,
    pub peace_db_conn: DbConnection<Peace>,
    pub bancho_state_rpc_client: BanchoStateRpcClient<Channel>,
    pub chat_rpc_client: ChatRpcClient<Channel>,
    pub geoip_service: DynGeoipService,
    pub users_repository: DynUsersRepository,
    pub bancho_state_service: DynBanchoStateService,
    pub chat_service: DynChatService,
    pub password_service: DynPasswordService,
    pub bancho_background_service: DynBanchoBackgroundService,
    pub bancho_background_service_config: BanchoBackgroundServiceConfigs,
    pub bancho_service: DynBanchoService,
    pub bancho_rpc: BanchoRpcImpl,
}

impl App {
    pub async fn initialize(cfg: Arc<BanchoConfig>) -> Self {
        let peace_db_conn = cfg
            .peace_db
            .connect()
            .await
            .expect("failed to connect peace db, please check.");

        let bancho_state_rpc_client = cfg.bancho_state.connect().await;

        let chat_rpc_client = cfg.chat.connect().await;

        let geoip_service =
            GeoipServiceBuilder::build::<GeoipServiceImpl, GeoipServiceRemote>(
                cfg.geo_db_path.as_deref(),
                Some(&cfg.geoip),
            )
            .await;

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn.clone()).into_service();

        let bancho_state_service = BanchoStateServiceRemote::from_client(
            bancho_state_rpc_client.clone(),
        )
        .into_service();

        let chat_service =
            ChatServiceRemote::from_client(chat_rpc_client.clone())
                .into_service();

        let password_service = PasswordServiceImpl::default();
        let password_cache_store = password_service.cache_store().clone();
        let password_service = password_service.into_service();

        let bancho_background_service =
            BanchoBackgroundServiceImpl::new(password_cache_store)
                .into_service();

        let bancho_background_service_config = BanchoBackgroundServiceConfigs {
            password_caches_recycle: PasswordCachesRecycleConfig::buid_with_cfg(
                &cfg.bancho_background_service_configs,
            ),
        };

        bancho_background_service
            .start_all(bancho_background_service_config.clone());

        let bancho_service = BanchoServiceImpl::new(
            users_repository.clone(),
            bancho_state_service.clone(),
            password_service.clone(),
            bancho_background_service.clone(),
            geoip_service.clone(),
            chat_service.clone(),
        )
        .into_service();

        let bancho_rpc = BanchoRpcImpl::new(bancho_service.clone());

        Self {
            cfg,
            peace_db_conn,
            bancho_state_rpc_client,
            chat_rpc_client,
            geoip_service,
            users_repository,
            bancho_state_service,
            chat_service,
            password_service,
            bancho_background_service,
            bancho_background_service_config,
            bancho_service,
            bancho_rpc,
        }
    }
}

#[async_trait]
impl RpcApplication for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn default_listen_addr(&self) -> Option<SocketAddr> {
        Some("127.0.0.1:5010".parse().unwrap())
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[BANCHO_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        configured_server.add_service(BanchoRpcServer::with_interceptor(
            self.bancho_rpc.clone(),
            client_ip,
        ))
    }
}
