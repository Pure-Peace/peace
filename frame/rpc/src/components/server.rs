use std::time::Duration;

use crate::{cfg::RpcFrameConfig, Application};
use once_cell::sync::OnceCell;
use tonic::transport::{server::Router, Server};
use tools::async_collections::{shutdown_signal, SignalHandle};

/// Start service.
pub async fn serve(app_cfg: impl Application) {
    let cfg = app_cfg.frame_cfg_arc();
    let svr = app_cfg.service(server(&cfg));

    #[cfg(feature = "reflection")]
    let svr = if cfg.reflection && app_cfg.service_descriptor().is_some() {
        add_reflection(svr, &app_cfg)
    } else {
        svr
    };

    #[cfg(feature = "admin_rpc")]
    let svr = if cfg.admin_rpc {
        svr.add_service(
            peace_pb::frame::logs::logs_rpc_server::LogsRpcServer::new(
                peace_logs::grpc::LogsRpcService::default(),
            ),
        )
    } else {
        svr
    };

    let _ = tokio::join!(launch_server(svr, &cfg), shutdown_signal(shutdown));
    warn!("!!! SERVER STOPPED !!!")
}

pub async fn launch_server(svr: Router, cfg: &RpcFrameConfig) {
    let handle = server_handle();
    info!(">> [gRPC SERVER] listening on: {}", addr(cfg));
    svr.serve_with_shutdown(cfg.addr, handle.wait_signal()).await.unwrap();
}

pub fn addr(cfg: &RpcFrameConfig) -> String {
    format!("{}://{}", if cfg.tls { "https" } else { "http" }, cfg.addr)
}

pub fn server(cfg: &RpcFrameConfig) -> Server {
    #[cfg(not(feature = "tls"))]
    let svr = Server::builder();

    #[cfg(feature = "tls")]
    let svr = if cfg.tls { tls_server(cfg) } else { Server::builder() };

    let svr = svr
        .accept_http1(cfg.accept_http1)
        .http2_adaptive_window(cfg.http2_adaptive_window)
        .http2_keepalive_interval(
            cfg.http2_keepalive_interval.map(Duration::from_secs),
        )
        .http2_keepalive_timeout(
            cfg.http2_keepalive_timeout.map(Duration::from_secs),
        )
        .initial_connection_window_size(cfg.initial_connection_window_size)
        .initial_stream_window_size(cfg.initial_stream_window_size)
        .max_concurrent_streams(cfg.max_concurrent_streams)
        .max_frame_size(cfg.max_frame_size)
        .tcp_keepalive(cfg.tcp_keepalive.map(Duration::from_secs))
        .tcp_nodelay(cfg.tcp_nodelay);

    let svr = if let Some(limit) = cfg.concurrency_limit_per_connection {
        svr.concurrency_limit_per_connection(limit)
    } else {
        svr
    };

    let svr = if let Some(timeout) = cfg.req_timeout {
        svr.timeout(Duration::from_secs(timeout))
    } else {
        svr
    };

    svr
}

#[cfg(feature = "tls")]
pub fn tls_server(cfg: &RpcFrameConfig) -> Server {
    let cert =
        std::fs::read(cfg.ssl_cert.as_ref().expect(
            "ERROR: tls: Please make sure `--ssl-cert` are passed in.",
        ))
        .unwrap();
    let key = std::fs::read(
        cfg.ssl_key
            .as_ref()
            .expect("ERROR: tls: Please make sure `--ssl-key` are passed in."),
    )
    .unwrap();

    let identity = tonic::transport::Identity::from_pem(cert, key);

    Server::builder()
        .tls_config(tonic::transport::ServerTlsConfig::new().identity(identity))
        .unwrap()
}

pub fn add_reflection(svr: Router, app_cfg: &impl Application) -> Router {
    svr.add_service(
        tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(
                app_cfg.service_descriptor().unwrap(),
            )
            .build()
            .unwrap(),
    )
}

pub fn server_handle() -> SignalHandle {
    static HANDLE: OnceCell<SignalHandle> = OnceCell::new();
    HANDLE.get_or_init(|| SignalHandle::new()).clone()
}

fn shutdown(s: &str) {
    warn!(">> [{}] Signal received, shutdown.", s);
    server_handle().trigger();
}
