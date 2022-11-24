use crate::components::{error::Error, router::AnyPathRouters};
use axum::{
    body::{Body, BoxBody},
    extract::Host,
    http::Request,
    response::{IntoResponse, Response},
    Router,
};
use tower::{load_shed, timeout, BoxError, ServiceExt};

/// Route `/` handler.
pub async fn app_root() -> Response {
    tools::pkg_metadata!().into_response()
}

/// Route `/*path` handler.
pub async fn any_path(
    Host(hostname): Host,
    mut req: Request<Body>,
    any_routers: AnyPathRouters,
) -> Response {
    // Fix `axum 0.6.0-rc5` `src/extract/matched_path.rs:146` debug_assert panic.
    req.extensions_mut().remove::<axum::extract::MatchedPath>();

    let result = match hostname.as_str() {
        "c.peace.local" | "osu.peace.local" => {
            call_router(any_routers.bancho, req).await
        },
        _ => return Error::NotFound.into(),
    };

    result.into_response()
}

pub async fn handle_404() -> Response {
    Error::NotFound.into()
}

pub async fn call_router(
    router: Router,
    req: Request<Body>,
) -> Response<BoxBody> {
    router.into_service().oneshot(req).await.into_response()
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
