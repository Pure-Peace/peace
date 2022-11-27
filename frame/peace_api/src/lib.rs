#[macro_use]
extern crate peace_logs;

mod components;

pub use crate::macro_impl_config as impl_config;
pub use components::*;

use axum::{body::Body, extract::Host, http::Request, Router};
use cfg::ApiFrameConfig;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use utoipa::openapi::OpenApi;

/// We can build app using peace_api,
/// just use [`Application`] and implement this trait for App.
pub trait Application: Clone + Send + Sync + 'static {
    /// App cfg should inherit [`ApiFrameConfig`], so this function is used to return it.
    fn frame_cfg(&self) -> &ApiFrameConfig;

    fn frame_cfg_arc(&self) -> Arc<ApiFrameConfig> {
        static FRAME_CFG: OnceCell<Arc<ApiFrameConfig>> = OnceCell::new();
        FRAME_CFG
            .get_or_init(|| Arc::new(self.frame_cfg().clone()))
            .clone()
    }

    /// Returns the [`Router`] for this app
    fn router(&self) -> Router;

    /// Returns the OpenApi documentation for this app.
    fn apidocs(&self) -> OpenApi;

    /// This is for `hostname routing`.
    ///
    /// Match the hostname with the specified service, and return a router,
    /// and the server will try to execute the corresponding implementation according to the path.
    fn match_hostname(&self, host: Host, req: &Request<Body>)
        -> Option<Router>;
}
