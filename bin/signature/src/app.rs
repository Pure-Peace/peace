use crate::SignatureRpcImpl;
use clap_serde_derive::ClapSerde;
use peace_pb::signature::{
    signature_rpc_server::SignatureRpcServer, SIGNATURE_DESCRIPTOR_SET,
};
use peace_rpc::{Application, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use peace_services::signature::SignatureServiceBuilder;
use std::{path::PathBuf, sync::Arc};
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

#[peace_config]
#[command(name = "signature", author, version, about, propagate_version = true)]
pub struct SignatureConfig {
    #[command(flatten)]
    pub runtime_cfg: RuntimeConfig,

    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,

    #[arg(long, short = 'P')]
    pub ed25519_private_key_path: Option<PathBuf>,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<SignatureConfig>,
}

impl App {
    pub fn new(cfg: Arc<SignatureConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Application for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[SIGNATURE_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        let signature_service = SignatureServiceBuilder::build(
            self.cfg.ed25519_private_key_path.as_ref().map(|path| {
                path.to_str()
                    .expect("failed to parse \"ed25519_private_key_path\"")
            }),
            None,
        )
        .await;

        let signature_rpc = SignatureRpcImpl::new(signature_service);

        configured_server.add_service(SignatureRpcServer::new(signature_rpc))
    }
}
