use crate::{components::router, ApiFrameConfig, WebApplication};
use axum::Router;
use axum_server::{AddrIncomingConfig, Handle};
use once_cell::sync::OnceCell;
use std::{net::SocketAddr, time::Duration};
use tools::async_collections::shutdown_signal;

/// Start service.
pub async fn serve(app_cfg: impl WebApplication) {
    tools::framework_info!();

    let cfg = app_cfg.frame_cfg_arc();
    let app = router::app(app_cfg).await;

    let config = AddrIncomingConfig::new()
        .tcp_nodelay(cfg.tcp_nodelay)
        .tcp_sleep_on_accept_errors(cfg.tcp_sleep_on_accept_errors)
        .tcp_keepalive(cfg.tcp_keepalive.map(Duration::from_secs))
        .tcp_keepalive_interval(
            cfg.tcp_keepalive_interval.map(Duration::from_secs),
        )
        .tcp_keepalive_retries(cfg.tcp_keepalive_retries)
        .build();

    print_api_docs(&cfg);
    #[cfg(feature = "tls")]
    if cfg.tls_config.tls {
        let https = tls::launch_https_server(app.clone(), &cfg, config.clone());
        if cfg.force_https {
            tokio::join!(
                tls::launch_ssl_redirect_server(&cfg),
                https,
                shutdown_signal(shutdown)
            );
        } else {
            tokio::join!(
                launch_http_server(app, &cfg, config),
                https,
                shutdown_signal(shutdown)
            );
        }
    } else {
        tokio::join!(
            launch_http_server(app, &cfg, config),
            shutdown_signal(shutdown)
        );
    }

    #[cfg(not(feature = "tls"))]
    tokio::join!(
        launch_http_server(app, &cfg, config),
        shutdown_signal(shutdown)
    );
    warn!("!!! SERVER STOPPED !!!")
}

pub fn server_handle() -> Handle {
    static HANDLE: OnceCell<Handle> = OnceCell::new();
    HANDLE.get_or_init(Handle::new).clone()
}

pub async fn launch_http_server(
    app: Router,
    cfg: &ApiFrameConfig,
    incoming_config: AddrIncomingConfig,
) {
    info!(">> [HTTP SERVER] listening on: http://{}", cfg.http_addr);
    axum_server::bind(cfg.http_addr)
        .handle(server_handle())
        .addr_incoming_config(incoming_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

pub fn print_api_docs(cfg: &ApiFrameConfig) {
    let addr = addr(cfg);
    info!(">> [Swagger UI]: {}{}", addr, cfg.swagger_path);
    info!(">> [openapi.json]: {}{}", addr, cfg.openapi_json);
}

pub fn addr(cfg: &ApiFrameConfig) -> String {
    #[cfg(feature = "tls")]
    if cfg.tls_config.tls {
        format!("https://{}", cfg.https_addr)
    } else {
        format!("http://{}", cfg.http_addr)
    }

    #[cfg(not(feature = "tls"))]
    format!("http://{}", cfg.http_addr)
}

#[cfg(feature = "tls")]
pub mod tls {
    use crate::{http::server_handle, ApiFrameConfig};
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
    pub async fn launch_ssl_redirect_server(cfg: &ApiFrameConfig) {
        let http_port = cfg.http_addr.port().to_string();
        let https_port = cfg.https_addr.port().to_string();

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
        info!(
            ">> [HTTP SERVER] (only redirect http to https) listening on: http://{}",
            cfg.http_addr
        );
        axum_server::bind(cfg.http_addr)
            .handle(server_handle())
            .serve(redirect.into_make_service())
            .await
            .unwrap();
    }

    pub async fn launch_https_server(
        app: Router,
        cfg: &ApiFrameConfig,
        incoming_config: AddrIncomingConfig,
    ) {
        let tls_config = RustlsConfig::from_pem_file(
            cfg.tls_config.ssl_cert.as_ref().expect(
                "ERROR: tls: Please make sure `--ssl-cert` are passed in.",
            ),
            cfg.tls_config.ssl_key.as_ref().expect(
                "ERROR: tls: Please make sure `--ssl-key` are passed in.",
            ),
        )
        .await
        .unwrap();

        info!(">> [HTTPS SERVER] listening on: https://{}", cfg.https_addr);
        axum_server::bind_rustls(cfg.https_addr, tls_config)
            .handle(server_handle())
            .addr_incoming_config(incoming_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    }
}

fn shutdown(s: &str) {
    warn!(">> [{}] Signal received, shutdown.", s);
    server_handle().shutdown();
}
