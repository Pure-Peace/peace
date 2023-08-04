use crate::SignatureRpcImpl;
use clap_serde_derive::ClapSerde;
use core_signature::*;
use pb_signature::{
    signature_rpc_server::SignatureRpcServer, SIGNATURE_DESCRIPTOR_SET,
};
use peace_rpc::{RpcApplication, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use std::{net::SocketAddr, sync::Arc};
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

/// PEACE Signature service
#[peace_config]
#[command(name = "signature", author, version, about, propagate_version = true)]
pub struct SignatureConfig {
    #[command(flatten)]
    pub runtime_cfg: RuntimeConfig,

    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,

    #[arg(long, short = 'P')]
    pub ed25519_private_key_path: Option<String>,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<SignatureConfig>,
    pub signature_service: DynSignatureService,
    pub signature_rpc: SignatureRpcImpl,
}

impl App {
    pub async fn initialize(cfg: Arc<SignatureConfig>) -> Self {
        let signature_service =
            SignatureServiceBuilder::build::<
                SignatureServiceImpl,
                SignatureServiceRemote,
            >(cfg.ed25519_private_key_path.as_deref(), None)
            .await;

        let signature_rpc = SignatureRpcImpl::new(signature_service.clone());

        Self { cfg, signature_service, signature_rpc }
    }
}

#[async_trait]
impl RpcApplication for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn default_listen_addr(&self) -> Option<SocketAddr> {
        Some("127.0.0.1:5014".parse().unwrap())
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[SIGNATURE_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        configured_server
            .add_service(SignatureRpcServer::new(self.signature_rpc.clone()))
    }
}
