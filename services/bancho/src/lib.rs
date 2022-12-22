pub mod cfg;
pub mod impls;
pub mod rpc;

use cfg::BanchoConfig;
use peace_pb::services::bancho::{
    bancho_rpc_server::BanchoRpcServer, BANCHO_DESCRIPTOR_SET,
};
use peace_rpc::{cfg::RpcFrameConfig, Application};
use rpc::Bancho;
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

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
        let svc = Bancho::default();
        configured_server.add_service(BanchoRpcServer::new(svc))
    }
}
