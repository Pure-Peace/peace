pub mod cmd;
pub mod error;
pub mod responder;
pub mod router;

#[cfg(feature = "tls")]
pub mod tls;

use crate::components::cmd::PeaceGatewayArgs;
use axum::{Router, Server};
use tokio::signal;

/// Start service.
pub async fn serve(args: &PeaceGatewayArgs) {
    let app = router::app(args);

    info!("\n\n{}\n", tools::pkg_metadata!());

    #[cfg(feature = "tls")]
    if args.tls {
        let https = tls::launch_https_server(app.clone(), args);
        if args.force_https {
            tokio::join!(tls::launch_ssl_redirect_server(args), https);
        } else {
            tokio::join!(launch_http_server(app, args), https);
        }
    } else {
        launch_http_server(app, args).await;
    }

    #[cfg(not(feature = "tls"))]
    launch_http_server(app, args).await;
}

pub async fn launch_http_server(app: Router, args: &PeaceGatewayArgs) {
    info!(">> [HTTP] listening on: {}", args.http_addr);
    Server::bind(&args.http_addr).serve(app.into_make_service()).await.unwrap();
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
