pub mod apidocs;
pub mod bancho;
pub mod cmd;

use apidocs::GatewayApiDocs;
use axum::{body::Body, extract::Host, http::Request, routing::get, Router};
use cmd::PeaceGatewayArgs;
use peace_api::{cmd::PeaceApiArgs, Application};
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
    fn frame_args(&self) -> &PeaceApiArgs {
        &self.args.api_framework_args
    }

    fn router(&self) -> Router {
        Router::new()
            .route("/", get(peace_api::responder::app_root))
            .nest("/bancho", bancho::routers::bancho_client_routes())
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
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
