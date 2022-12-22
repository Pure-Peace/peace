use crate::{
    Application,
    {cfg::ApiFrameConfig, responder, responder::shutdown_server},
};
use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::Host,
    http::Request,
    routing::{any, delete},
    Router,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use utoipa::openapi::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// App router with some middleware.
pub async fn app(app: impl Application) -> Router {
    let cfg = app.frame_cfg_arc();
    app_router(app)
        .await
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(responder::handle_error))
                .load_shed()
                .concurrency_limit(cfg.concurrency_limit)
                .timeout(Duration::from_secs(cfg.req_timeout))
                .layer(TraceLayer::new_for_http()),
        )
        .fallback(responder::handle_404)
}

pub fn openapi_router(mut openapi: OpenApi, cfg: &ApiFrameConfig) -> Router {
    if !cfg.admin_endpoints {
        openapi = remove_admin_routes(openapi)
    }
    SwaggerUi::new(cfg.swagger_path.clone())
        .url(cfg.openapi_json.clone(), openapi)
        .into()
}

pub fn remove_admin_routes(mut openapi: OpenApi) -> OpenApi {
    let admin_pathes = openapi
        .paths
        .paths
        .keys()
        .filter(|k| k.starts_with("/admin"))
        .map(|k| k.to_string())
        .collect::<Vec<String>>();

    for key in admin_pathes {
        openapi.paths.paths.remove(&key);
    }

    openapi
}

/// The `admin_routers` provides some api endpoints for managing the server,
/// such as setting the log level and stopping the server.
///
/// You can pass in admin_token to add a layer of Authorization authentication (using Bearer).
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
pub async fn app_router(app: impl Application) -> Router {
    let cfg = app.frame_cfg();
    let mut router =
        openapi_router(app.apidocs(), cfg).merge(app.router().await);

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
