use crate::BanchoStateRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_pb::bancho_state::{
    bancho_state_rpc_server::BanchoStateRpcServer, BANCHO_STATE_DESCRIPTOR_SET,
};
use peace_rpc::{RpcApplication, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::{
    bancho_state::*,
    rpc_config::SignatureRpcConfig,
    signature::{
        DynSignatureService, SignatureServiceBuilder, SignatureServiceImpl,
        SignatureServiceRemote,
    },
    IntoService,
};
use std::{net::SocketAddr, sync::Arc};
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

    #[command(flatten)]
    pub bancho_state_snapshot: CliBanchoStateServiceSnapshopConfigs,
}

/// The BanchoState application struct.
#[derive(Clone)]
pub struct App {
    /// The configuration for the BanchoState application.
    pub cfg: Arc<BanchoStateConfig>,
    pub user_sessions_service: DynUserSessionsService,
    pub signature_service: DynSignatureService,
    pub bancho_state_background_service: DynBanchoStateBackgroundService,
    pub bancho_state_background_service_config:
        BanchoStateBackgroundServiceConfigs,
    pub bancho_state_service: DynBanchoStateService,
    pub bancho_state_rpc: BanchoStateRpcImpl,
}

impl App {
    /// Create a new BanchoState application instance with the provided
    /// configuration.
    pub async fn initialize(cfg: Arc<BanchoStateConfig>) -> Self {
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

        let bancho_state_background_service =
            Arc::new(BanchoStateBackgroundServiceImpl::new(
                user_sessions_service.clone(),
            ));

        let bancho_state_background_service_config =
            BanchoStateBackgroundServiceConfigs::with_cfg(
                &cfg.bancho_state_background_service_configs,
            );

        bancho_state_background_service
            .start_all(bancho_state_background_service_config.clone());

        // Create a new BanchoState instance.
        let bancho_state_rpc =
            BanchoStateRpcImpl::new(bancho_state_service.clone());

        Self {
            cfg,
            user_sessions_service,
            signature_service,
            bancho_state_background_service,
            bancho_state_background_service_config,
            bancho_state_service,
            bancho_state_rpc,
        }
    }
}

#[async_trait]
impl RpcApplication for App {
    /// Get the RPC frame configuration for the BanchoState application.
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn default_listen_addr(&self) -> Option<SocketAddr> {
        Some("127.0.0.1:5011".parse().unwrap())
    }

    /// Get the service descriptors for the BanchoState application.
    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[BANCHO_STATE_DESCRIPTOR_SET])
    }

    /// Start the BanchoState application and return a Router.
    async fn service(&self, mut configured_server: Server) -> Router {
        // Add the BanchoState service to the server.
        configured_server.add_service(BanchoStateRpcServer::new(
            self.bancho_state_rpc.clone(),
        ))
    }
}
