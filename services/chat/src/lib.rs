#[macro_use]
extern crate peace_rpc;

pub mod rpc;

use clap_serde_derive::ClapSerde;
use peace_dal::db::peace::PeaceDbConfig;
use peace_pb::services::chat_rpc::{
    chat_rpc_server::ChatRpcServer, CHAT_DESCRIPTOR_SET,
};
use peace_rpc::{Application, RpcFrameConfig};
use rpc::ChatService;
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

/// Command Line Interface (CLI) for Chat service.
#[peace_config]
#[command(name = "chat", author, version, about, propagate_version = true)]
pub struct ChatServiceConfig {
    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,

    #[command(flatten)]
    pub peace_db: PeaceDbConfig,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<ChatServiceConfig>,
}

impl App {
    pub fn new(cfg: Arc<ChatServiceConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Application for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[CHAT_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        let svc = ChatService::default();
        configured_server.add_service(ChatRpcServer::new(svc))
    }
}
