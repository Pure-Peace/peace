#[macro_use]
extern crate peace_logs;

mod components;

pub use components::*;
pub use peace_cfg::{
    macro_define_rpc_client_config as define_rpc_client_config,
    macro_impl_config as impl_config, peace_config, ParseConfig,
    RpcClientConfig, SingletonConfig,
};
pub use peace_rpc_error::*;

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::{net::SocketAddr, sync::Arc};
use thiserror::Error;
use tonic::metadata::errors::ToStrError;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};
use tower_layer::Identity;

pub type DescriptorBuf<'a> = &'a [u8];

/// We can build app using `peace_rpc`,
/// just use [`Application`] and implement this trait for your App.
#[async_trait]
pub trait RpcApplication: Clone + Send + Sync + 'static {
    /// App cfg should inherit [`RpcFrameConfig`], so this function is used to
    /// return it.
    fn frame_cfg(&self) -> &RpcFrameConfig;

    fn frame_cfg_arc(&self) -> Arc<RpcFrameConfig> {
        static FRAME_CFG: OnceCell<Arc<RpcFrameConfig>> = OnceCell::new();
        FRAME_CFG.get_or_init(|| Arc::new(self.frame_cfg().clone())).clone()
    }

    /// In order to implement reflection, the descriptor needs to be returned in
    /// this method.
    #[cfg(feature = "reflection")]
    fn service_descriptors(&self) -> Option<&[DescriptorBuf]>;

    fn default_listen_addr(&self) -> Option<SocketAddr> {
        None
    }

    async fn service(&self, configured_server: Server) -> Router<Identity>;
}

pub trait RpcError<'de>: Serialize + Deserialize<'de> + Display {}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing metadata entry")]
    MissingMetadata,

    #[error("invalid status code set")]
    InvalidStatusCode(tonic::Status),

    #[error("could not parse metadata to string")]
    MetadataParseError(#[from] ToStrError),

    #[error("serde json error")]
    JsonError(#[from] serde_json::Error),
}
