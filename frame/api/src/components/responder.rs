use crate::{
    components::{error::Error, http::server_handle},
    WebApplication,
};
use axum::{
    body::{Body, BoxBody},
    extract::{Host, Path},
    http::Request,
    response::{IntoResponse, Response},
    Router,
};
use std::time::Duration;
use tower::{load_shed, timeout, BoxError, ServiceExt};

/// Route `/` handler.
pub async fn app_root() -> Response {
    tools::pkg_metadata!().into_response()
}

/// Stop the server within a specified time `grace_period_secs`.
#[utoipa::path(
    delete,
    context_path = "/admin",
    path = "/server/shutdown/{grace_period_secs}",
    tag = "admin",
    responses(
        (status = 200, description = "Success"),
    ),
    params(
        ("grace_period_secs" = u64, Path, description = "shutdown grace period seconds", example = "3")
    ),
    security(("admin_token" = []))
)]
pub async fn shutdown_server(Path(grace_period_secs): Path<u64>) -> Response {
    warn!(
        "!!! [api::shutdown_server]: The server will stop in [{}] seconds !!!",
        grace_period_secs
    );
    server_handle()
        .graceful_shutdown(Some(Duration::from_secs(grace_period_secs)));
    "ok".into_response()
}

/// Route `/*path` handler.
pub async fn any_path(
    host: Host,
    mut req: Request<Body>,
    app: impl WebApplication,
) -> Response {
    // Fix `axum 0.6.0-rc5` `src/extract/matched_path.rs:146` debug_assert
    // panic.
    req.extensions_mut().remove::<axum::extract::MatchedPath>();

    match app.match_hostname(host, &req).await {
        Some(router) => call_router(router, req).await,
        None => Error::NotFound.into_response(),
    }
}

pub async fn handle_404() -> Response {
    Error::NotFound.into()
}

pub async fn call_router(
    router: Router,
    req: Request<Body>,
) -> Response<BoxBody> {
    router.oneshot(req).await.into_response()
}

pub async fn handle_error(error: BoxError) -> Error {
    if error.is::<timeout::error::Elapsed>() {
        return Error::Timeout;
    }

    if error.is::<load_shed::error::Overloaded>() {
        return Error::Unavailable;
    }

    anyhow::anyhow!("Unhandled internal error: {:?}", error).into()
}
