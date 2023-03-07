use clap_serde_derive::ClapSerde;
use peace_pb::bancho_state::{
    bancho_state_rpc_server::BanchoStateRpcServer, BANCHO_STATE_DESCRIPTOR_SET,
};
use peace_rpc::{Application, RpcFrameConfig};
use peace_services::bancho_state::{
    BanchoStateBackgroundServiceImpl, BanchoStateServiceImpl,
    UserSessionsServiceImpl,
};
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

use crate::BanchoStateRpcImpl;

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
        let user_sessions = UserSessionsServiceImpl::default().into_service();

        let bancho_state_background_service =
            BanchoStateBackgroundServiceImpl::new(
                user_sessions.user_sessions().clone(),
            )
            .into_service();

        bancho_state_background_service.start_all();

        let bancho_state_service = BanchoStateServiceImpl::local(
            user_sessions,
            bancho_state_background_service,
        )
        .into_service();

        // Create a new BanchoState instance.
        let bancho_state_rpc = BanchoStateRpcImpl::new(bancho_state_service);

        // Add the BanchoState service to the server.
        configured_server
            .add_service(BanchoStateRpcServer::new(bancho_state_rpc))
    }
}
