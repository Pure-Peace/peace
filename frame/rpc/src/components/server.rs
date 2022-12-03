use std::time::Duration;

use crate::{cfg::RpcFrameConfig, Application};
use once_cell::sync::OnceCell;
use tonic::{
    metadata::MetadataValue,
    transport::{server::Router, Server},
};
use tools::async_collections::{shutdown_signal, SignalHandle};

#[cfg(feature = "admin_rpc")]
use peace_pb::frame::logs::logs_rpc_server::LogsRpcServer;

/// Start service.
pub async fn serve(app_cfg: impl Application) {
    let cfg = app_cfg.frame_cfg_arc();
    let mut svr = app_cfg.service(server(&cfg));

    #[cfg(feature = "reflection")]
    if cfg.reflection {
        svr = add_reflection(svr, &app_cfg)
    };

    #[cfg(feature = "admin_rpc")]
    if cfg.admin_rpc {
        let svc = peace_logs::grpc::LogsRpcService::default();
        if let Some(token) = cfg.admin_token.clone() {
            svr = svr.add_service(LogsRpcServer::with_interceptor(
                svc,
                move |req| {
                    let token: MetadataValue<_> =
                        format!("Bearer {token}").parse().unwrap();
                    crate::interceptor::check_auth(req, token)
                },
            ))
        } else {
            svr = svr.add_service(LogsRpcServer::new(svc))
        }
    };

    let _ = tokio::join!(launch_server(svr, &cfg), shutdown_signal(shutdown));
    warn!("!!! SERVER STOPPED !!!")
}

pub async fn launch_server(svr: Router, cfg: &RpcFrameConfig) {
    let handle = server_handle();
    info!(">> [gRPC SERVER] listening on: {}", addr(cfg));

    #[cfg(unix)]
    if let Some(path) = cfg.uds {
        tokio::fs::create_dir_all(
            std::path::Path::new(&path).parent().unwrap(),
        )
        .await
        .unwrap();

        let uds = tokio::net::UnixListener::bind(path).unwrap();
        let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds);
        svr.serve_with_incoming_shutdown(uds_stream, handle.wait_signal())
            .await
            .unwrap();
    } else {
        svr.serve_with_shutdown(cfg.addr, handle.wait_signal()).await.unwrap();
    }

    #[cfg(not(unix))]
    svr.serve_with_shutdown(cfg.addr, handle.wait_signal()).await.unwrap();
}

pub fn addr(cfg: &RpcFrameConfig) -> String {
    #[cfg(unix)]
    if let Some(path) = cfg.uds {
        return format!("{}", path);
    }

    format!("{}://{}", if cfg.tls { "https" } else { "http" }, cfg.addr)
}

pub fn server(cfg: &RpcFrameConfig) -> Server {
    #[cfg(not(feature = "tls"))]
    let svr = Server::builder();

    #[cfg(feature = "tls")]
    let svr = if cfg.tls { tls_server(cfg) } else { Server::builder() };

    let mut svr = svr
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

    if let Some(limit) = cfg.concurrency_limit_per_connection {
        svr = svr.concurrency_limit_per_connection(limit)
    };

    if let Some(timeout) = cfg.req_timeout {
        svr = svr.timeout(Duration::from_secs(timeout))
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
    let mut reflection = tonic_reflection::server::Builder::configure();

    #[cfg(feature = "admin_rpc")]
    if app_cfg.frame_cfg().admin_rpc {
        reflection = reflection.register_encoded_file_descriptor_set(
            peace_pb::frame::logs::LOGS_DESCRIPTOR_SET,
        );
    };

    if let Some(descriptors) = app_cfg.service_descriptors() {
        for i in descriptors {
            reflection = reflection.register_encoded_file_descriptor_set(i);
        }
    }

    svr.add_service(reflection.build().unwrap())
}

pub fn server_handle() -> SignalHandle {
    static HANDLE: OnceCell<SignalHandle> = OnceCell::new();
    HANDLE.get_or_init(|| SignalHandle::new()).clone()
}

fn shutdown(s: &str) {
    warn!(">> [{}] Signal received, shutdown.", s);
    server_handle().trigger();
}
