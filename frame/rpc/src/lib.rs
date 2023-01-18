#[macro_use]
extern crate peace_logs;

mod components;

pub use components::*;
pub use peace_cfg::{
    macro_impl_config as impl_config, peace_config, ParseConfig,
    SingletonConfig,
};

use once_cell::sync::OnceCell;
use std::sync::Arc;
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};
use tower_layer::Identity;

pub type DescriptorBuf<'a> = &'a [u8];

/// We can build app using `peace_rpc`,
/// just use [`Application`] and implement this trait for your App.
#[async_trait]
pub trait Application: Clone + Send + Sync + 'static {
    /// App cfg should inherit [`RpcFrameConfig`], so this function is used to return it.
    fn frame_cfg(&self) -> &RpcFrameConfig;

    fn frame_cfg_arc(&self) -> Arc<RpcFrameConfig> {
        static FRAME_CFG: OnceCell<Arc<RpcFrameConfig>> = OnceCell::new();
        FRAME_CFG.get_or_init(|| Arc::new(self.frame_cfg().clone())).clone()
    }

    /// In order to implement reflection, the descriptor needs to be returned in this method.
    #[cfg(feature = "reflection")]
    fn service_descriptors(&self) -> Option<&[DescriptorBuf]>;

    async fn service(&self, configured_server: Server) -> Router<Identity>;
}
