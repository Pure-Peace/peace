use crate::{components::router, WebApplication};
use axum::Router;
use axum_server::{AddrIncomingConfig, Handle};
use once_cell::sync::OnceCell;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tools::async_collections::shutdown_signal;

pub const DEFAULT_BINDING_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub const DEFAULT_HTTP_PORT: u16 = 8000;
pub const DEFAULT_HTTPS_PORT: u16 = 443;

pub const DEFAULT_HTTP_ADDR: SocketAddr =
    SocketAddr::new(DEFAULT_BINDING_IP, DEFAULT_HTTP_PORT);
pub const DEFAULT_HTTPS_ADDR: SocketAddr =
    SocketAddr::new(DEFAULT_BINDING_IP, DEFAULT_HTTPS_PORT);

const BANNER: &str = r#"
    ____  _________   ____________               _
   / __ \/ ____/   | / ____/ ____/  ____ _____  (_)
  / /_/ / __/ / /| |/ /   / __/    / __ `/ __ \/ /
 / ____/ /___/ ___ / /___/ /___   / /_/ / /_/ / /
/_/   /_____/_/  |_\____/_____/   \__,_/ .___/_/
                                      /_/
"#;

/// Start service.
pub async fn serve(app: impl WebApplication) {
    tools::framework_info!(BANNER);
    println!(">> Starting service...");

    let cfg = app.frame_cfg_arc();

    let http_addr = cfg
        .http_addr
        .unwrap_or(app.default_http_addr().unwrap_or(DEFAULT_HTTP_ADDR));

    let https_addr = cfg
        .https_addr
        .unwrap_or(app.default_https_addr().unwrap_or(DEFAULT_HTTPS_ADDR));

    let app = router::app(app).await;

    let incoming_config = AddrIncomingConfig::new()
        .tcp_nodelay(cfg.tcp_nodelay)
        .tcp_sleep_on_accept_errors(cfg.tcp_sleep_on_accept_errors)
        .tcp_keepalive(cfg.tcp_keepalive.map(Duration::from_secs))
        .tcp_keepalive_interval(
            cfg.tcp_keepalive_interval.map(Duration::from_secs),
        )
        .tcp_keepalive_retries(cfg.tcp_keepalive_retries)
        .build();

    print_api_docs(
        cfg.tls_config.tls,
        http_addr,
        https_addr,
        cfg.swagger_path.as_str(),
        cfg.openapi_json.as_str(),
    );

    #[cfg(feature = "tls")]
    if cfg.tls_config.tls {
        let https = tls::launch_https_server(
            app.clone(),
            https_addr,
            cfg.tls_config.ssl_cert.as_ref(),
            cfg.tls_config.ssl_key.as_ref(),
            incoming_config.clone(),
        );

        if cfg.force_https {
            tokio::join!(
                tls::launch_ssl_redirect_server(http_addr, https_addr),
                https,
                shutdown_signal(shutdown)
            );
        } else {
            tokio::join!(
                launch_http_server(app, http_addr, incoming_config),
                https,
                shutdown_signal(shutdown)
            );
        }
    } else {
        tokio::join!(
            launch_http_server(app, http_addr, incoming_config),
            shutdown_signal(shutdown)
        );
    }

    #[cfg(not(feature = "tls"))]
    tokio::join!(
        launch_http_server(app, http_addr, incoming_config),
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
    http_addr: SocketAddr,
    incoming_config: AddrIncomingConfig,
) {
    info!(">> [HTTP SERVER] listening on: http://{}", http_addr);
    axum_server::bind(http_addr)
        .handle(server_handle())
        .addr_incoming_config(incoming_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

pub fn print_api_docs(
    tls: bool,
    http_addr: SocketAddr,
    https_addr: SocketAddr,
    swagger_path: &str,
    openapi_json: &str,
) {
    let addr = addr(tls, http_addr, https_addr);
    info!(">> [Swagger UI]: {}{}", addr, swagger_path);
    info!(">> [openapi.json]: {}{}", addr, openapi_json);
}

pub fn addr(
    tls: bool,
    http_addr: SocketAddr,
    https_addr: SocketAddr,
) -> String {
    #[cfg(feature = "tls")]
    if tls {
        format!("https://{}", https_addr)
    } else {
        format!("http://{}", http_addr)
    }

    #[cfg(not(feature = "tls"))]
    format!("http://{}", http_addr)
}

#[cfg(feature = "tls")]
pub mod tls {
    use crate::http::server_handle;
    use axum::{
        extract::Host,
        handler::HandlerWithoutStateExt,
        http::{StatusCode, Uri},
        response::Redirect,
        BoxError, Router,
    };
    use axum_server::{tls_rustls::RustlsConfig, AddrIncomingConfig};
    use std::{net::SocketAddr, path::PathBuf};

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
    pub async fn launch_ssl_redirect_server(
        http_addr: SocketAddr,
        https_addr: SocketAddr,
    ) {
        let http_port = http_addr.port().to_string();
        let https_port = https_addr.port().to_string();

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
            http_addr
        );
        axum_server::bind(http_addr)
            .handle(server_handle())
            .serve(redirect.into_make_service())
            .await
            .unwrap();
    }

    pub async fn launch_https_server(
        app: Router,
        https_addr: SocketAddr,
        ssl_cert: Option<&PathBuf>,
        ssl_key: Option<&PathBuf>,
        incoming_config: AddrIncomingConfig,
    ) {
        let tls_config = RustlsConfig::from_pem_file(
            ssl_cert.expect(
                "ERROR: tls: Please make sure `--ssl-cert` are passed in.",
            ),
            ssl_key.expect(
                "ERROR: tls: Please make sure `--ssl-key` are passed in.",
            ),
        )
        .await
        .unwrap();

        info!(">> [HTTPS SERVER] listening on: https://{}", https_addr);
        axum_server::bind_rustls(https_addr, tls_config)
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
