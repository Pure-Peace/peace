use crate::ChatRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_db::{
    peace::{Peace, PeaceDbConfig},
    DbConfig, DbConnection,
};
use peace_pb::chat::{chat_rpc_server::ChatRpcServer, CHAT_DESCRIPTOR_SET};
use peace_repositories::users::{DynUsersRepository, UsersRepositoryImpl};
use peace_rpc::{RpcApplication, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::chat::{
    ChatBackgroundServiceConfigs, ChatBackgroundServiceImpl,
    ChatServiceDumpLoader, CliChatBackgroundServiceConfigs,
    DynChatBackgroundService, DynChatService,
};
use std::{net::SocketAddr, sync::Arc};
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

    #[default("./chat.dump".to_owned())]
    #[arg(long, default_value = "./chat.dump")]
    pub chat_dump_path: String,

    #[arg(long)]
    pub chat_save_dump: bool,

    #[arg(long)]
    pub chat_load_dump: bool,

    #[default(300)]
    #[arg(long, default_value = "300")]
    pub chat_dump_expries: u64,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<ChatServiceConfig>,
    pub peace_db_conn: DbConnection<Peace>,
    pub users_repository: DynUsersRepository,
    pub chat_service: DynChatService,
    pub chat_background_service: DynChatBackgroundService,
    pub chat_background_service_config: ChatBackgroundServiceConfigs,
    pub chat_rpc: ChatRpcImpl,
}

impl App {
    pub async fn initialize(cfg: Arc<ChatServiceConfig>) -> Self {
        let peace_db_conn = cfg
            .peace_db
            .connect()
            .await
            .expect("failed to connect peace db, please check.");

        let users_repository =
            UsersRepositoryImpl::new(peace_db_conn.clone()).into_service();

        let chat_service = ChatServiceDumpLoader::load(
            cfg.chat_load_dump,
            &cfg.chat_dump_path,
            cfg.chat_dump_expries,
            users_repository.clone(),
        )
        .await
        .into_service();

        let chat_background_service =
            ChatBackgroundServiceImpl::new(chat_service.clone()).into_service();

        let chat_background_service_config =
            ChatBackgroundServiceConfigs::with_cfg(
                &cfg.chat_background_service_configs,
            );

        chat_service
            .load_public_channels()
            .await
            .expect("Failed to load public channels");

        chat_background_service
            .start_all(chat_background_service_config.clone());

        let chat_rpc = ChatRpcImpl::new(chat_service.clone());

        Self {
            cfg,
            peace_db_conn,
            users_repository,
            chat_service,
            chat_background_service,
            chat_background_service_config,
            chat_rpc,
        }
    }
}

#[async_trait]
impl RpcApplication for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn default_listen_addr(&self) -> Option<SocketAddr> {
        Some("127.0.0.1:5012".parse().unwrap())
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[CHAT_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        configured_server.add_service(ChatRpcServer::new(self.chat_rpc.clone()))
    }
}
