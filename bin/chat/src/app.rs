use crate::ChatRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_pb::chat::{chat_rpc_server::ChatRpcServer, CHAT_DESCRIPTOR_SET};
use peace_rpc::{RpcApplication, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::chat::ChatServiceImpl;
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

/// PEACE Chat gRPC service
#[peace_config]
#[command(name = "chat", author, version, about, propagate_version = true)]
pub struct ChatServiceConfig {
    #[command(flatten)]
    pub runtime_cfg: RuntimeConfig,

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
impl RpcApplication for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[CHAT_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        let peace_db_conn = self
            .cfg
            .peace_db
            .connect()
            .await
            .expect("failed to connect peace db, please check.");

        let chat_service = ChatServiceImpl::new(peace_db_conn).into_service();

        chat_service.load_public_channels().await.expect("debugging");

        let chat_rpc = ChatRpcImpl::new(chat_service);

        configured_server.add_service(ChatRpcServer::new(chat_rpc))
    }
}
