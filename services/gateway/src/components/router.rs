use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use axum::{
    error_handling::HandleErrorLayer,
    routing::{any, get},
    Router as AxumRouter,
};
use matchit::Router;
use peace_logs::api::admin_routers;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    bancho,
    components::{
        cmd::PeaceGatewayArgs,
        responder::{self, handle_error, handler_404, HandlerWrapper},
    },
};

pub struct HandlerRouter {
    inner: Router<HandlerWrapper>,
}

impl HandlerRouter {
    pub fn new() -> Self {
        HandlerRouter { inner: Router::new() }
    }

    pub fn route(mut self, route: &str, dest: HandlerWrapper) -> Self {
        self.inner.insert(route, dest).unwrap();
        self
    }
}

impl Deref for HandlerRouter {
    type Target = Router<HandlerWrapper>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for HandlerRouter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn app(args: &PeaceGatewayArgs) -> AxumRouter {
    app_router(args)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(args.concurrency_limit)
                .timeout(Duration::from_secs(args.req_timeout))
                .layer(TraceLayer::new_for_http()),
        )
        .fallback(handler_404)
}

pub fn app_router(args: &PeaceGatewayArgs) -> AxumRouter {
    let router = AxumRouter::new()
        .route("/", get(responder::app_root))
        .nest("/bancho", bancho::routers::bancho_client_routes());

    let router = if args.admin_api {
        router.nest("/admin", admin_routers(args.admin_token.as_deref()))
    } else {
        router
    };

    if args.hostname_routing {
        router.route("/*path", any(responder::any_path))
    } else {
        router
    }
}
