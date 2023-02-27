use bancho_state::repositorys::*;
use bancho_state::service::*;
use clap_serde_derive::ClapSerde;
use peace_pb::services::bancho_state_rpc::{
    bancho_state_rpc_server::BanchoStateRpcServer, BANCHO_STATE_DESCRIPTOR_SET,
};
use peace_rpc::{Application, RpcFrameConfig};
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

#[derive(Clone)]
pub struct BanchoState {
    pub app_state_repository: DynAppStateRepository,
    pub packets_repository: DynPacketsRepository,
    pub background_repository: DynBackgroundServiceRepository,
    pub sessions_repository: DynSessionsRepository,
}

impl BanchoState {
    pub fn new(
        app_state_repository: DynAppStateRepository,
        packets_repository: DynPacketsRepository,
        background_repository: DynBackgroundServiceRepository,
        sessions_repository: DynSessionsRepository,
    ) -> BanchoState {
        Self {
            app_state_repository,
            packets_repository,
            background_repository,
            sessions_repository,
        }
    }
}

#[peace_config]
#[command(
    name = "bancho_state",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct BanchoStateConfig {
    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,
}

/// The BanchoState application struct.
#[derive(Clone)]
pub struct App {
    /// The configuration for the BanchoState application.
    pub cfg: Arc<BanchoStateConfig>,
}

impl App {
    /// Create a new BanchoState application instance with the provided configuration.
    pub fn new(cfg: Arc<BanchoStateConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Application for App {
    /// Get the RPC frame configuration for the BanchoState application.
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    /// Get the service descriptors for the BanchoState application.
    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[BANCHO_STATE_DESCRIPTOR_SET])
    }

    /// Start the BanchoState application and return a Router.
    async fn service(&self, mut configured_server: Server) -> Router {
        let app_state = Arc::new(AppState::default()) as DynAppStateRepository;
        let packets_repository =
            Arc::new(BanchoStatePacketsRepository::default())
                as DynPacketsRepository;
        let background_repository =
            Arc::new(BanchoStateBackgroundServiceRepository::default())
                as DynBackgroundServiceRepository;
        let sessions_repository =
            Arc::new(BanchoStateSessionsRepository::default())
                as DynSessionsRepository;

        // Create a new BanchoState instance.
        let bancho_state = BanchoState::new(
            app_state,
            packets_repository,
            background_repository,
            sessions_repository,
        );

        // Add the BanchoState service to the server.
        configured_server.add_service(BanchoStateRpcServer::new(bancho_state))
    }
}
