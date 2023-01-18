#[macro_use]
extern crate peace_rpc;

pub mod impls;
pub mod rpc;

use clap_serde_derive::ClapSerde;
use peace_pb::services::bancho::{
    bancho_rpc_server::BanchoRpcServer, BANCHO_DESCRIPTOR_SET,
};
use peace_rpc::{interceptor::client_ip, Application, RpcFrameConfig};
use rpc::Bancho;
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

/// Command Line Interface (CLI) for Bancho service.
#[peace_config]
#[command(name = "bancho", author, version, about, propagate_version = true)]
pub struct BanchoConfig {
    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,
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
        let bancho = Bancho::default();
        configured_server
            .add_service(BanchoRpcServer::with_interceptor(bancho, client_ip))
    }
}
