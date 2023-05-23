#[macro_use]
extern crate peace_logs;
#[macro_use]
extern crate anyhow;

mod components;
pub mod extractors;

pub use components::*;
pub use peace_cfg::{
    macro_define_rpc_client_config as define_rpc_client_config,
    macro_impl_config as impl_config, peace_config, ParseConfig,
    RpcClientConfig, SingletonConfig,
};

use axum::{async_trait, body::Body, extract::Host, http::Request, Router};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use utoipa::openapi::OpenApi;

/// We can build app using `peace_api`,
/// just use [`Application`] and implement this trait for App.
#[async_trait]
pub trait Application: Clone + Send + Sync + 'static {
    /// App cfg should inherit [`ApiFrameConfig`], so this function is used to
    /// return it.
    fn frame_cfg(&self) -> &ApiFrameConfig;

    fn frame_cfg_arc(&self) -> Arc<ApiFrameConfig> {
        static FRAME_CFG: OnceCell<Arc<ApiFrameConfig>> = OnceCell::new();
        FRAME_CFG.get_or_init(|| Arc::new(self.frame_cfg().clone())).clone()
    }

    /// Returns the [`Router`] for this app
    async fn router<T: Clone + Sync + Send + 'static>(&self)
        -> Router<T, Body>;

    /// Returns the OpenApi documentation for this app.
    fn apidocs(&self) -> OpenApi;

    /// This is for `hostname routing`.
    ///
    /// Match the hostname with the specified service, and return a router,
    /// and the server will try to execute the corresponding implementation
    /// according to the path.
    async fn match_hostname(
        &self,
        _host: Host,
        _req: &Request<Body>,
    ) -> Option<Router> {
        None
    }
}
