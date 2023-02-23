#[macro_use]
extern crate peace_rpc;
#[macro_use]
extern crate peace_logs;

mod components;
pub use components::*;
pub mod rpc;

use clap_serde_derive::ClapSerde;
use peace_pb::services::bancho_state_rpc::{
    bancho_state_rpc_server::BanchoStateRpcServer, BANCHO_STATE_DESCRIPTOR_SET,
};
use peace_rpc::{Application, RpcFrameConfig};
use rpc::BanchoState;
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

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

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<BanchoStateConfig>,
}

impl App {
    pub fn new(cfg: Arc<BanchoStateConfig>) -> Self {
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
        let bancho_state = BanchoState::default();

        bancho_state.start_background_service();

        configured_server.add_service(BanchoStateRpcServer::new(bancho_state))
    }
}
