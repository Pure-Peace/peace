pub mod cfg;
pub mod rpc;

use cfg::DbConfig;
use peace_pb::services::db::db_rpc_server::DbRpcServer;
use peace_rpc::{cfg::RpcFrameConfig, Application};
use rpc::Db;
use std::sync::Arc;
use tonic::transport::{server::Router, Server};

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<DbConfig>,
}

impl App {
    pub fn new(cfg: Arc<DbConfig>) -> Self {
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
        let svc = Db::default();
        configured_server.add_service(DbRpcServer::new(svc))
    }
}
