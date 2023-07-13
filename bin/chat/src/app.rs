use crate::ChatRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_pb::chat::{chat_rpc_server::ChatRpcServer, CHAT_DESCRIPTOR_SET};
use peace_repositories::users::UsersRepositoryImpl;
use peace_rpc::{RpcApplication, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::chat::{
    ChatBackgroundService, ChatBackgroundServiceConfigs,
    ChatBackgroundServiceImpl, ChatService, ChatServiceImpl,
    CliChatBackgroundServiceConfigs,
};
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

    #[command(flatten)]
    pub chat_background_service_configs: CliChatBackgroundServiceConfigs,
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

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn).into_service();

        let chat_service = Arc::new(ChatServiceImpl::new(users_repository));

        let chat_background_service =
            Arc::new(ChatBackgroundServiceImpl::new(chat_service.clone()));

        let chat_background_service_config =
            ChatBackgroundServiceConfigs::with_cfg(
                &self.cfg.chat_background_service_configs,
            );

        chat_service.load_public_channels().await.expect("debugging");

        chat_background_service.start_all(chat_background_service_config);

        let chat_rpc = ChatRpcImpl::new(chat_service);

        configured_server.add_service(ChatRpcServer::new(chat_rpc))
    }
}
