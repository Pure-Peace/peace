use crate::cmd::PeaceGatewayArgs;
use crate::routes;

use tokio::signal;
use tower::{load_shed, timeout, BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

use std::{borrow::Cow, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

pub fn app(args: &PeaceGatewayArgs) -> Router {
    Router::new()
        .route("/", get(routes::root))
        .route("/bancho", post(routes::bancho))
        .nest(
            "/admin",
            peace_logs::api::admin_routers(args.admin_token.as_deref()),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(args.concurrency_limit)
                .timeout(Duration::from_secs(args.req_timeout))
                .layer(TraceLayer::new_for_http()),
        )
        .fallback(handler_404.into_service())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    let s = tokio::select! {
        _ = ctrl_c => "CONTROL_C",
        _ = terminate => "TERMINATE",
    };

    warn!("[{}] Signal received, shutdown.", s);
}
