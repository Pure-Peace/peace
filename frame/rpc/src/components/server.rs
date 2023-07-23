use crate::{RpcApplication, RpcFrameConfig};
use once_cell::sync::OnceCell;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    time::Duration,
};
use tonic::{
    metadata::MetadataValue,
    transport::{server::Router, Server},
};
use tools::async_collections::{shutdown_signal, SignalHandle};

#[cfg(feature = "admin_endpoints")]
use peace_pb::logs::logs_rpc_server::LogsRpcServer;

pub const DEFAULT_BINDING_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub const DEFAULT_RPC_PORT: u16 = 5010;

pub const DEFAULT_RPC_ADDR: SocketAddr =
    SocketAddr::new(DEFAULT_BINDING_IP, DEFAULT_RPC_PORT);

/// Start service.
pub async fn serve(app: impl RpcApplication) {
    tools::framework_info!();

    // Get the configuration from the application.
    let cfg = app.frame_cfg_arc();

    // Create a server with the given configuration.
    let mut svr = app.service(server(&cfg)).await;

    // If the 'reflection' feature is enabled and the configuration specifies
    // it, add reflection to the server.
    #[cfg(feature = "reflection")]
    if cfg.rpc_reflection {
        svr = add_reflection(svr, &app)
    };

    // If the 'admin_endpoints' feature is enabled and the configuration
    // specifies it, add the logs service to the server with the appropriate
    // authorization interceptor.
    #[cfg(feature = "admin_endpoints")]
    if cfg.rpc_admin_endpoints {
        let svc = peace_logs::grpc::LogsRpcService::default();
        if let Some(token) = cfg.rpc_admin_token.clone() {
            // Create the logs service with the authorization interceptor.
            svr = svr.add_service(LogsRpcServer::with_interceptor(
                svc,
                move |req| {
                    let token: MetadataValue<_> =
                        format!("Bearer {token}").parse().unwrap();
                    crate::interceptor::admin_endpoints_authorization(
                        req, token,
                    )
                },
            ))
        } else {
            // Create the logs service without an interceptor.
            svr = svr.add_service(LogsRpcServer::new(svc))
        }
    };

    // Launch the server and wait for the shutdown signal.
    let _ = tokio::join!(
        launch_server(
            svr,
            cfg.rpc_tls_config.tls,
            cfg.rpc_addr.unwrap_or(
                app.default_listen_addr().unwrap_or(DEFAULT_RPC_ADDR)
            ),
            #[cfg(unix)]
            cfg.rpc_uds.as_ref()
        ),
        shutdown_signal(shutdown)
    );

    // Log that the server has stopped.
    warn!("!!! SERVER STOPPED !!!")
}

/// Launches the gRPC server and serves incoming requests.
///
/// * svr - The Router instance that contains the gRPC services to be served.
pub async fn launch_server(
    svr: Router,
    tls: bool,
    rpc_addr: SocketAddr,
    #[cfg(unix)] rpc_uds: Option<&PathBuf>,
) {
    // Create a server handle for handling graceful shutdown.
    let handle = server_handle();
    // Log the address that the server is listening on.
    info!(
        ">> [gRPC SERVER] listening on: {}",
        addr(
            tls,
            rpc_addr,
            #[cfg(unix)]
            rpc_uds
        )
    );

    // Check if the Unix Domain Socket option is enabled.
    #[cfg(unix)]
    if let Some(path) = rpc_uds {
        // Create the parent directory of the UDS if it doesn't exist.
        tokio::fs::create_dir_all(std::path::Path::new(path).parent().unwrap())
            .await
            .unwrap();

        // Bind the UDS listener to the specified path.
        let uds = tokio::net::UnixListener::bind(path).unwrap();
        // Create a stream of incoming connections from the UDS listener.
        let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds);
        // Serve incoming requests with the UDS stream and wait for a shutdown
        // signal.
        svr.serve_with_incoming_shutdown(uds_stream, handle.wait_signal())
            .await
            .unwrap();
    } else {
        // Serve incoming requests with the specified address and wait for a
        // shutdown signal.
        svr.serve_with_shutdown(rpc_addr, handle.wait_signal()).await.unwrap();
    }

    // If the Unix Domain Socket option is not enabled, serve incoming requests
    // with the specified address and wait for a shutdown signal.
    #[cfg(not(unix))]
    svr.serve_with_shutdown(rpc_addr, handle.wait_signal()).await.unwrap();
}

/// This function generates the address string for the gRPC server
///
///
/// # Returns
///
/// A string containing the address of the gRPC server, formatted as
/// protocol://address:port or path for Unix Domain Sockets
pub fn addr(
    tls: bool,
    rpc_addr: SocketAddr,
    #[cfg(unix)] rpc_uds: Option<&PathBuf>,
) -> String {
    #[cfg(unix)]
    // If the server is using Unix Domain Sockets, return the path as the
    // address
    if let Some(path) = &rpc_uds {
        return format!("{}", path.to_string_lossy());
    }

    // If the server is not using Unix Domain Sockets, return the address and
    // protocol as the address
    format!("{}://{}", if tls { "https" } else { "http" }, rpc_addr)
}

/// Returns a `Server` based on the provided `RpcFrameConfig` configuration.
pub fn server(cfg: &RpcFrameConfig) -> Server {
    #[cfg(not(feature = "tls"))] // check if the feature "tls" is not enabled
    let svr = Server::builder(); // create a new server builder

    #[cfg(feature = "tls")] // check if the feature "tls" is enabled
    let svr = if cfg.rpc_tls_config.tls {
        // if the config specifies to use tls
        tls_server(
            cfg.rpc_tls_config.ssl_cert.as_ref(),
            cfg.rpc_tls_config.ssl_key.as_ref(),
        ) // create a tls server
    } else {
        Server::builder() // create a new server builder
    };

    let mut svr = svr // store the created server builder
        .accept_http1(cfg.rpc_accept_http1) // set whether to accept HTTP/1 requests
        .http2_adaptive_window(cfg.rpc_http2_adaptive_window) // set the adaptive window size for HTTP/2 streams
        .http2_keepalive_interval(
            cfg.rpc_http2_keepalive_interval.map(Duration::from_secs),
        ) // set the interval for HTTP/2 keepalive pings
        .http2_keepalive_timeout(
            cfg.rpc_http2_keepalive_timeout.map(Duration::from_secs),
        ) // set the timeout for HTTP/2 keepalive pings
        .initial_connection_window_size(cfg.rpc_initial_connection_window_size) // set the initial window size for new HTTP/2 connections
        .initial_stream_window_size(cfg.rpc_initial_stream_window_size) // set the initial window size for new HTTP/2 streams
        .max_concurrent_streams(cfg.rpc_max_concurrent_streams) // set the maximum number of concurrent streams for HTTP/2 connections
        .max_frame_size(cfg.rpc_max_frame_size) // set the maximum frame size for HTTP/2 streams
        .tcp_keepalive(cfg.rpc_tcp_keepalive.map(Duration::from_secs)) // set the keepalive interval for TCP connections
        .tcp_nodelay(cfg.rpc_tcp_nodelay); // set whether to enable TCP_NODELAY

    if let Some(limit) = cfg.rpc_concurrency_limit_per_connection {
        // if the configuration specifies a concurrency limit per connection
        svr = svr.concurrency_limit_per_connection(limit) // set the concurrency
                                                          // limit per connection
    };

    if let Some(timeout) = cfg.rpc_req_timeout {
        // if the configuration specifies a request timeout
        svr = svr.timeout(Duration::from_secs(timeout)) // set the request
                                                        // timeout
    };

    svr // return the created server
}

/// Returns a Tonic `Server` with TLS configuration.
///
/// # Arguments
///
/// * `cfg` - A reference to the `RpcFrameConfig` struct that contains
///   configuration details related to the RPC server.
///
/// # Panics
///
/// This function will panic if `--ssl-cert` or `--ssl-key` options are not
/// passed in.
#[cfg(feature = "tls")]
pub fn tls_server(
    ssl_cert: Option<&PathBuf>,
    ssl_key: Option<&PathBuf>,
) -> Server {
    // Read the SSL certificate file into a byte buffer.
    let cert = std::fs::read(
        ssl_cert
            .expect("ERROR: tls: Please make sure `--ssl-cert` are passed in."),
    )
    .unwrap();

    // Read the SSL key file into a byte buffer.
    let key = std::fs::read(
        ssl_key
            .expect("ERROR: tls: Please make sure `--ssl-key` are passed in."),
    )
    .unwrap();

    // Create a new identity using the certificate and key byte buffers.
    let identity = tonic::transport::Identity::from_pem(cert, key);

    // Create a new server builder with TLS configuration using the created
    // identity.
    Server::builder()
        .tls_config(tonic::transport::ServerTlsConfig::new().identity(identity))
        .unwrap()
}

/// Adds reflection to a `Router` using the provided `Application`
/// configuration.
///
/// # Arguments
///
/// * `svr` - The `Router` to add the reflection to.
/// * `app` - The `Application`.
///
/// # Returns
///
/// The `Router` with the reflection added.
pub fn add_reflection(svr: Router, app: &impl RpcApplication) -> Router {
    // Create a reflection builder
    let mut reflection = tonic_reflection::server::Builder::configure();

    // If admin endpoints are enabled, register the logs descriptor set
    #[cfg(feature = "admin_endpoints")]
    if app.frame_cfg().rpc_admin_endpoints {
        reflection = reflection.register_encoded_file_descriptor_set(
            peace_pb::logs::LOGS_DESCRIPTOR_SET,
        );
    };

    // Register the encoded file descriptor sets for each service in the
    // application configuration
    if let Some(descriptors) = app.service_descriptors() {
        for i in descriptors {
            reflection = reflection.register_encoded_file_descriptor_set(i);
        }
    }

    // Add the reflection service to the Router
    svr.add_service(reflection.build().unwrap())
}

/// Returns the singleton `SignalHandle` used for server shutdown signals.
///
/// # Returns
///
/// The `SignalHandle` used for server shutdown signals.
pub fn server_handle() -> SignalHandle {
    // Create a static once cell to store the SignalHandle
    static HANDLE: OnceCell<SignalHandle> = OnceCell::new();

    // Return the SignalHandle, initializing it if necessary
    HANDLE.get_or_init(SignalHandle::new).clone()
}

/// Shuts down the server with the given message.
///
/// # Arguments
///
/// * `s` - The message to display when shutting down.
fn shutdown(s: &str) {
    // Log the shutdown message
    warn!(">> [{}] Signal received, shutdown.", s);

    // Trigger the server shutdown signal
    server_handle().trigger();
}
