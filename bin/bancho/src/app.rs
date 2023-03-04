use crate::BanchoRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_pb::{
    bancho_rpc::bancho_rpc_server::BanchoRpcServer,
    bancho_state_rpc::{self, BANCHO_STATE_DESCRIPTOR_SET},
};
use peace_repositories::users::UsersRepositoryImpl;
use peace_rpc::{
    interceptor::client_ip, Application, RpcClientConfig, RpcFrameConfig,
};
use peace_services::{
    bancho::{
        BanchoBackgroundServiceImpl, BanchoServiceImpl, PasswordServiceImpl,
    },
    bancho_state::BanchoStateServiceImpl,
};
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

define_rpc_client_config!(
    service_name: bancho_state_rpc,
    config_name: BanchoStateRpcConfig
);

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

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn).into_service();

        let bancho_state_service =
            BanchoStateServiceImpl::remote(bancho_state_rpc_client)
                .into_service();

        let password_service = PasswordServiceImpl::default().into_service();

        let bancho_background_service =
            BanchoBackgroundServiceImpl::new(password_service.cache().clone())
                .into_service();

        bancho_background_service.start_all();

        let bancho_service = BanchoServiceImpl::local(
            users_repository,
            bancho_state_service,
            password_service,
            bancho_background_service,
        )
        .into_service();

        let bancho_rpc = BanchoRpcImpl::new(bancho_service);

        configured_server.add_service(BanchoRpcServer::with_interceptor(
            bancho_rpc, client_ip,
        ))
    }
}
