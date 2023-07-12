use crate::BanchoStateRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_pb::bancho_state::{
    bancho_state_rpc_server::BanchoStateRpcServer, BANCHO_STATE_DESCRIPTOR_SET,
};
use peace_rpc::{RpcApplication, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::{
    bancho_state::{
        BanchoStateBackgroundService, BanchoStateBackgroundServiceConfigs,
        BanchoStateBackgroundServiceImpl, BanchoStateServiceImpl,
        CliBanchoStateBackgroundServiceConfigs, UserSessionsServiceImpl,
    },
    rpc_config::SignatureRpcConfig,
    signature::{
        SignatureServiceBuilder, SignatureServiceImpl, SignatureServiceRemote,
    },
    IntoService,
};
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

/// BanchoState gRPC service
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
    pub runtime_cfg: RuntimeConfig,

    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,

    #[command(flatten)]
    pub bancho_state_background_service_configs:
        CliBanchoStateBackgroundServiceConfigs,

    #[command(flatten)]
    pub signature_rpc_cfg: SignatureRpcConfig,

    #[arg(long)]
    pub ed25519_private_key_path: Option<String>,
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
impl RpcApplication for App {
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
        let user_session_service = Arc::new(UserSessionsServiceImpl::default());

        let signature_service = SignatureServiceBuilder::build::<
            SignatureServiceImpl,
            SignatureServiceRemote,
        >(
            self.cfg.ed25519_private_key_path.as_deref(),
            Some(&self.cfg.signature_rpc_cfg),
        )
        .await;

        let bancho_state_background_service = Arc::new(
            BanchoStateBackgroundServiceImpl::new(user_session_service.clone()),
        );

        let bancho_state_background_service_config =
            BanchoStateBackgroundServiceConfigs::with_cfg(
                &self.cfg.bancho_state_background_service_configs,
            );

        bancho_state_background_service
            .start_all(bancho_state_background_service_config);

        let bancho_state_service = BanchoStateServiceImpl::new(
            user_session_service,
            signature_service,
        )
        .into_service();

        // Create a new BanchoState instance.
        let bancho_state_rpc = BanchoStateRpcImpl::new(bancho_state_service);

        // Add the BanchoState service to the server.
        configured_server
            .add_service(BanchoStateRpcServer::new(bancho_state_rpc))
    }
}
