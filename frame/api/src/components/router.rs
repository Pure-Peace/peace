use crate::{
    responder, responder::shutdown_server, PeaceApiAdminEndpointsDocs,
    WebApplication,
};
use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::Host,
    http::Request,
    routing::{any, delete},
    Router,
};
use peace_logs::Level;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// App router with some middleware.
pub async fn app(app: impl WebApplication) -> Router {
    let cfg = app.frame_cfg_arc();
    app_router(app)
        .await
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(responder::handle_error))
                .load_shed()
                .concurrency_limit(cfg.concurrency_limit)
                .timeout(Duration::from_secs(cfg.req_timeout))
                .layer(
                    TraceLayer::new_for_http().on_failure(
                        DefaultOnFailure::new().level(Level::DEBUG),
                    ),
                ),
        )
        .fallback(responder::handle_404)
}

/// The `admin_routers` provides some api endpoints for managing the server,
/// such as setting the log level and stopping the server.
///
/// You can pass in admin_token to add a layer of Authorization authentication
/// (using Bearer).
pub fn admin_routers(admin_token: Option<&str>) -> Router {
    peace_logs::api::admin_routers(
        admin_token,
        Some(Router::new().route(
            "/admin/server/shutdown/:grace_period_secs",
            delete(shutdown_server),
        )),
    )
}

/// App router
pub async fn app_router(app: impl WebApplication) -> Router {
    let cfg = app.frame_cfg();
    let mut router =
        Into::<Router>::into(SwaggerUi::new(cfg.swagger_path.clone()).url(
            cfg.openapi_json.clone(),
            {
                let mut docs = app.apidocs();
                if cfg.admin_endpoints {
                    docs.merge(PeaceApiAdminEndpointsDocs::openapi())
                }
                docs
            },
        ))
        .merge(app.router().await);

    if cfg.admin_endpoints {
        router = router.merge(admin_routers(cfg.admin_token.as_deref()))
    };

    if cfg.hostname_routing {
        router = router.route(
            "/*path",
            any(move |host: Host, req: Request<Body>| {
                responder::any_path(host, req, app)
            }),
        )
    };

    router
}
