pub mod apidocs;
pub mod bancho;
pub mod cmd;

use apidocs::GatewayApiDocs;
use axum::{body::Body, extract::Host, http::Request, routing::get, Router};
use cmd::PeaceGatewayArgs;
use peace_api::components::{cmd::PeaceApiArgs, router::Application};
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(Clone)]
pub struct App {
    pub args: Arc<PeaceGatewayArgs>,
}

impl App {
    pub fn new(args: Arc<PeaceGatewayArgs>) -> Self {
        Self { args }
    }
}

impl Application for App {
    fn framework_args(&self) -> Arc<PeaceApiArgs> {
        Arc::new(self.args.api_framework_args.clone())
    }

    fn router(&self) -> Router {
        Router::new()
            .route("/", get(peace_api::components::responder::app_root))
            .nest("/bancho", bancho::routers::bancho_client_routes())
    }

    fn openapi(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::openapi()
    }

    fn match_hostname(
        &self,
        Host(hostname): Host,
        req: &Request<Body>,
    ) -> Option<Router> {
        match hostname {
            n if self.args.bancho_hostname.contains(&n) => {
                Some(bancho::routers::bancho_client_routes())
            },
            _ => None,
        }
    }
}
