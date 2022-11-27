pub mod apidocs;
pub mod bancho;
pub mod cfg;

use apidocs::GatewayApiDocs;
use axum::{body::Body, extract::Host, http::Request, routing::get, Router};
use cfg::GatewayConfig;
use peace_api::{cfg::ApiFrameConfig, Application};
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<GatewayConfig>,
}

impl App {
    pub fn new(cfg: Arc<GatewayConfig>) -> Self {
        Self { cfg }
    }
}

impl Application for App {
    fn frame_cfg(&self) -> &ApiFrameConfig {
        &self.cfg.frame_cfg
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
            n if self.cfg.bancho_hostname.contains(&n) => {
                Some(bancho::routers::bancho_client_routes())
            },
            _ => None,
        }
    }
}
