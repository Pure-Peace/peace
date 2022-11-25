use crate::components::{cmd::PeaceGatewayArgs, router};
use axum::Router;
use axum_server::AddrIncomingConfig;
use std::{net::SocketAddr, time::Duration};
use tokio::signal;


/// Start service.
pub async fn serve(args: &PeaceGatewayArgs) {
    let app = router::app(args);

    let config = AddrIncomingConfig::new()
        .tcp_nodelay(args.tcp_nodelay)
        .tcp_sleep_on_accept_errors(args.tcp_sleep_on_accept_errors)
        .tcp_keepalive(args.tcp_keepalive.map(|i| Duration::from_secs(i)))
        .tcp_keepalive_interval(
            args.tcp_keepalive_interval.map(|i| Duration::from_secs(i)),
        )
        .tcp_keepalive_retries(args.tcp_keepalive_retries)
        .build();

    info!("\n\n{}\n", tools::pkg_metadata!());

    #[cfg(feature = "tls")]
    if args.tls {
        let https = tls::launch_https_server(app.clone(), args, config.clone());
        if args.force_https {
            tokio::join!(tls::launch_ssl_redirect_server(args), https);
        } else {
            tokio::join!(launch_http_server(app, args, config), https);
        }
    } else {
        launch_http_server(app, args, config).await;
    }

    #[cfg(not(feature = "tls"))]
    launch_http_server(app, args, config).await;
}

pub async fn launch_http_server(
    app: Router,
    args: &PeaceGatewayArgs,
    incoming_config: AddrIncomingConfig,
) {
    info!(">> [HTTP] listening on: {}", args.http_addr);
    axum_server::bind(args.http_addr)
        .addr_incoming_config(incoming_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
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

#[cfg(feature = "tls")]
pub mod tls {
    use crate::components::cmd::PeaceGatewayArgs;
    use axum::{
        extract::Host,
        handler::HandlerWithoutStateExt,
        http::{StatusCode, Uri},
        response::Redirect,
        BoxError, Router,
    };
    use axum_server::{tls_rustls::RustlsConfig, AddrIncomingConfig};
    use std::net::SocketAddr;

    /// Redirect `http` to `https`.
    pub fn redirect_replace(
        host: String,
        uri: Uri,
        http_port: &str,
        https_port: &str,
    ) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(http_port, https_port);
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    /// Start server that redirects `http` to `https`.
    pub async fn launch_ssl_redirect_server(args: &PeaceGatewayArgs) {
        let http_port = args.http_addr.port().to_string();
        let https_port = args.https_addr.port().to_string();

        let redirect = move |Host(host): Host, uri: Uri| async move {
            match redirect_replace(host, uri, &http_port, &https_port) {
                Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
                Err(error) => {
                    warn!(%error, "failed to convert URI to HTTPS");
                    Err(StatusCode::BAD_REQUEST)
                },
            }
        };

        info!(">> Force https enabled");
        info!(">> [HTTP] (only redirect http to https) listening on: {}", args.http_addr);
        axum_server::bind(args.http_addr)
            .serve(redirect.into_make_service())
            .await
            .unwrap();
    }

    pub async fn launch_https_server(
        app: Router,
        args: &PeaceGatewayArgs,
        incoming_config: AddrIncomingConfig,
    ) {
        let tls_config = RustlsConfig::from_pem_file(
            args.ssl_cert.as_ref().expect(
                "ERROR: tls: Please make sure `--ssl-cert` are passed in.",
            ),
            args.ssl_key.as_ref().expect(
                "ERROR: tls: Please make sure `--ssl-key` are passed in.",
            ),
        )
        .await
        .unwrap();

        info!(">> [HTTPS] listening on: {}", args.https_addr);
        axum_server::bind_rustls(args.https_addr, tls_config)
            .addr_incoming_config(incoming_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    }
}
