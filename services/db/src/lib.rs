pub mod cfg;
pub mod rpc;

use cfg::DbServiceConfig;
use peace_pb::services::peace_db::{
    peace_db_rpc_server::PeaceDbRpcServer, PEACE_DB_DESCRIPTOR_SET,
};
use peace_rpc::{cfg::RpcFrameConfig, Application};
use rpc::peace_db::PeaceDbService;
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<DbServiceConfig>,
}

impl App {
    pub fn new(cfg: Arc<DbServiceConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Application for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[PEACE_DB_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        let svc = PeaceDbService::default();
        configured_server.add_service(PeaceDbRpcServer::new(svc))
    }
}
