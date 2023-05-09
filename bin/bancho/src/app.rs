use crate::BanchoRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_pb::{
    bancho::{bancho_rpc_server::BanchoRpcServer, BANCHO_DESCRIPTOR_SET},
    bancho_state, chat, geoip,
};
use peace_repositories::users::UsersRepositoryImpl;
use peace_rpc::{
    interceptor::client_ip, Application, RpcClientConfig, RpcFrameConfig,
};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::{
    bancho::{
        BanchoBackgroundServiceConfigs, BanchoBackgroundServiceImpl,
        BanchoServiceImpl, CliBanchoBackgroundServiceConfigs,
        PasswordCachesRecycleConfig, PasswordServiceImpl,
    },
    bancho_state::BanchoStateServiceRemote,
    chat::ChatServiceRemote,
    geoip::GeoipServiceBuilder,
};
use std::{path::PathBuf, sync::Arc};
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

define_rpc_client_config!(
    service_name: bancho_state,
    config_name: BanchoStateRpcConfig
);

define_rpc_client_config!(
    service_name: geoip,
    config_name: GeoipRpcConfig,
    default_uri: "http://127.0.0.1:12346"
);

define_rpc_client_config!(
    service_name: chat,
    config_name: ChatRpcConfig,
    default_uri: "http://127.0.0.1:12347"
);

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
    pub geo_db_path: Option<PathBuf>,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<BanchoConfig>,
}

impl App {
    pub fn new(cfg: Arc<BanchoConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Application for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[BANCHO_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        let peace_db_conn = self
            .cfg
            .peace_db
            .connect()
            .await
            .expect("failed to connect peace db, please check.");

        let bancho_state_rpc_client =
            self.cfg.bancho_state.connect_client().await.unwrap_or_else(|err| {
                error!("Unable to connect to the bancho_state gRPC service, please make sure the service is started.");
                panic!("{}", err)
            });

        let chat_rpc_client =
            self.cfg.chat.connect_client().await.unwrap_or_else(|err| {
                error!("Unable to connect to the chat gRPC service, please make sure the service is started.");
                panic!("{}", err)
            });

        let geoip_service = GeoipServiceBuilder::build(
            self.cfg.geo_db_path.as_ref().map(|path| {
                path.to_str().expect("failed to parse geo_db_path")
            }),
            Some(&self.cfg.geoip),
        )
        .await;

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn).into_service();

        let bancho_state_service =
            BanchoStateServiceRemote::new(bancho_state_rpc_client)
                .into_service();

        let chat_service =
            ChatServiceRemote::new(chat_rpc_client).into_service();

        let password_service = PasswordServiceImpl::default().into_service();

        let bancho_background_service =
            BanchoBackgroundServiceImpl::new(password_service.cache().clone())
                .into_service();

        let bancho_background_service_config = BanchoBackgroundServiceConfigs {
            password_caches_recycle: PasswordCachesRecycleConfig::build(
                self.cfg
                    .bancho_background_service_configs
                    .password_caches_recycle_deactive_secs,
                self.cfg
                    .bancho_background_service_configs
                    .password_caches_recycle_interval_secs,
            ),
        };

        bancho_background_service.start_all(bancho_background_service_config);

        let bancho_service = BanchoServiceImpl::local(
            users_repository,
            bancho_state_service,
            password_service,
            bancho_background_service,
            geoip_service,
            chat_service,
        )
        .into_service();

        let bancho_rpc = BanchoRpcImpl::new(bancho_service);

        configured_server.add_service(BanchoRpcServer::with_interceptor(
            bancho_rpc, client_ip,
        ))
    }
}
