use crate::{
    bancho,
    components::{cmd::PeaceGatewayArgs, openapi::GatewayApiDocs, responder},
};
use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::Host,
    http::Request,
    routing::{any, get},
    Router,
};
use peace_logs::api::admin_routers;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AnyPathRouters {
    pub bancho: Router<()>,
}

/// App router with some middleware.
pub fn app(args: &PeaceGatewayArgs) -> Router {
    app_router(args)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(responder::handle_error))
                .load_shed()
                .concurrency_limit(args.concurrency_limit)
                .timeout(Duration::from_secs(args.req_timeout))
                .layer(TraceLayer::new_for_http()),
        )
        .fallback(responder::handle_404)
}

/// App router
pub fn app_router(args: &PeaceGatewayArgs) -> Router {
    let router = Router::new()
        .merge(
            SwaggerUi::new(args.swagger_path.clone())
                .url(args.openapi_json.clone(), GatewayApiDocs::openapi()),
        )
        .route("/", get(responder::app_root))
        .nest("/bancho", bancho::routers::bancho_client_routes());

    let router = if args.admin_api {
        router.merge(admin_routers(args.admin_token.as_deref()))
    } else {
        router
    };

    if args.hostname_routing {
        let any_routers =
            AnyPathRouters { bancho: bancho::routers::bancho_client_routes() };
        router.route(
            "/*path",
            any(|host: Host, req: Request<Body>| {
                responder::any_path(host, req, any_routers)
            }),
        )
    } else {
        router
    }
}
