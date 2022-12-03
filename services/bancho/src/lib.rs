pub mod cfg;
pub mod impls;
pub mod rpc;

use cfg::BanchoConfig;
use peace_pb::services::bancho::bancho_rpc_server::BanchoRpcServer;
use peace_rpc::{cfg::RpcFrameConfig, Application};
use rpc::Bancho;
use std::sync::Arc;
use tonic::transport::{server::Router, Server};

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<BanchoConfig>,
}

impl App {
    pub fn new(cfg: Arc<BanchoConfig>) -> Self {
        Self { cfg }
    }
}

impl Application for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn service_descriptor(&self) -> Option<&[u8]> {
        todo!()
    }

    fn service(&self, mut configured_server: Server) -> Router {
        let svc = Bancho::default();
        configured_server.add_service(BanchoRpcServer::new(svc))
    }
}
