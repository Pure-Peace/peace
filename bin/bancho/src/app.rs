use clap_serde_derive::ClapSerde;
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_pb::{
    bancho_rpc::bancho_rpc_server::BanchoRpcServer,
    bancho_state_rpc::{self, BANCHO_STATE_DESCRIPTOR_SET},
};
use peace_repositories::users::{DynUsersRepository, UsersRepositoryImpl};
use peace_rpc::{
    interceptor::client_ip, Application, RpcClientConfig, RpcFrameConfig,
};
use peace_services::{
    bancho::service::{
        BanchoServiceImpl, BanchoServiceLocal, DynBanchoService,
    },
    bancho_state::service::{
        BanchoStateServiceImpl, BanchoStateServiceRemote, DynBanchoStateService,
    },
};
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

#[derive(Clone)]
pub struct Bancho {
    pub bancho_service: DynBanchoService,
}

impl Bancho {
    pub fn new(bancho_service: DynBanchoService) -> Bancho {
        Self { bancho_service }
    }
}

define_rpc_client_config!(
    service_name: bancho_state_rpc,
    config_name: BanchoStateRpcConfig
);

/// Command Line Interface (CLI) for Bancho service.
#[peace_config]
#[command(name = "bancho", author, version, about, propagate_version = true)]
pub struct BanchoConfig {
    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,

    #[command(flatten)]
    pub peace_db: PeaceDbConfig,

    #[command(flatten)]
    pub bancho_state: BanchoStateRpcConfig,
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
        Some(&[BANCHO_STATE_DESCRIPTOR_SET])
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

        let users_repository = Arc::new(UsersRepositoryImpl::new(peace_db_conn))
            as DynUsersRepository;
        let bancho_state_service = Arc::new(BanchoStateServiceImpl::Remote(
            BanchoStateServiceRemote::new(bancho_state_rpc_client),
        )) as DynBanchoStateService;

        let bancho_service = Arc::new(BanchoServiceImpl::Local(
            BanchoServiceLocal::new(users_repository, bancho_state_service),
        )) as DynBanchoService;

        let bancho = Bancho::new(bancho_service);

        configured_server
            .add_service(BanchoRpcServer::with_interceptor(bancho, client_ip))
    }
}
