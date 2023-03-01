use clap_serde_derive::ClapSerde;
use peace_pb::bancho_state_rpc::{
    bancho_state_rpc_server::BanchoStateRpcServer, BANCHO_STATE_DESCRIPTOR_SET,
};
use peace_rpc::{Application, RpcFrameConfig};
use peace_services::bancho_state::{
    BackgroundServiceImpl, BanchoStateServiceImpl, BanchoStateServiceLocal,
    DynBackgroundService, DynBanchoStateService,
};
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

#[derive(Clone)]
pub struct BanchoState {
    pub bancho_state_service: DynBanchoStateService,
    pub background_service: DynBackgroundService,
}

impl BanchoState {
    pub fn new(
        bancho_state_service: DynBanchoStateService,
        background_service: DynBackgroundService,
    ) -> BanchoState {
        Self { bancho_state_service, background_service }
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
    /// Create a new BanchoState application instance with the provided
    /// configuration.
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
        let bancho_state_service = Arc::new(BanchoStateServiceImpl::Local(
            BanchoStateServiceLocal::default(),
        )) as DynBanchoStateService;
        let background_service =
            Arc::new(BackgroundServiceImpl::default()) as DynBackgroundService;

        // Create a new BanchoState instance.
        let bancho_state =
            BanchoState::new(bancho_state_service, background_service);

        // Add the BanchoState service to the server.
        configured_server.add_service(BanchoStateRpcServer::new(bancho_state))
    }
}
