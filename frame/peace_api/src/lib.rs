#[macro_use]
extern crate peace_logs;

mod components;

pub use crate::macro_impl_args as impl_args;
pub use components::*;

use axum::{body::Body, extract::Host, http::Request, Router};
use cmd::PeaceApiArgs;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use utoipa::openapi::OpenApi;

/// We can build app using peace_api,
/// just use [`Application`] and implement this trait for App.
pub trait Application: Clone + Send + Sync + 'static {
    /// App args should inherit [`PeaceApiArgs`], so this function is used to return it.
    fn frame_args(&self) -> &PeaceApiArgs;

    fn frame_args_arc(&self) -> Arc<PeaceApiArgs> {
        static FRAME_ARGS: OnceCell<Arc<PeaceApiArgs>> = OnceCell::new();
        FRAME_ARGS
            .get_or_init(|| Arc::new(self.frame_args().clone()))
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
