use crate::ChatRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_db::{peace::PeaceDbConfig, DbConfig};
use peace_pb::{
    bancho_state,
    chat::{chat_rpc_server::ChatRpcServer, CHAT_DESCRIPTOR_SET},
};
use peace_rpc::{Application, RpcClientConfig, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::{
    bancho_state::BanchoStateServiceRemote,
    chat::{ChannelServiceImpl, ChatServiceImpl},
};
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

define_rpc_client_config!(
    service_name: bancho_state,
    config_name: BanchoStateRpcConfig
);

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
    pub bancho_state: BanchoStateRpcConfig,
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
        let peace_db_conn = self
            .cfg
            .peace_db
            .connect()
            .await
            .expect("failed to connect peace db, please check.");

        let bancho_state_rpc_client =
            self.cfg.bancho_state.connect_client().await.unwrap_or_else(|err| {
                error!("Unable to connect to the bancho_state gRPC service, please make sure the service is started.");
                panic!("{}", err)
            });

        let bancho_state_service =
            BanchoStateServiceRemote::new(bancho_state_rpc_client)
                .into_service();

        let channel_service =
            ChannelServiceImpl::new(peace_db_conn).into_service();

        channel_service.initialize_public_channels().await;

        let chat_service =
            ChatServiceImpl::new(channel_service, bancho_state_service)
                .into_service();

        let chat_rpc = ChatRpcImpl::new(chat_service);

        configured_server.add_service(ChatRpcServer::new(chat_rpc))
    }
}
