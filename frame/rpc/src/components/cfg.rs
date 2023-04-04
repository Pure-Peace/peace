use clap_serde_derive::ClapSerde;
use peace_cfg::TlsConfig;
use peace_cfg::{impl_config, peace_config, SingletonConfig};
use peace_logs::LoggingConfigArgs;
use std::{default::Default, net::SocketAddr, ops::Deref, path::PathBuf};

/// Basic configuration items for `peace-rpc` framework.
#[peace_config]
pub struct RpcFrameConfig {
    /// Logging configurations.
    #[clap(flatten)]
    pub logging: LoggingConfigArgs,

    #[clap(flatten)]
    pub rpc: RpcServiceConfig,
}

impl Deref for RpcFrameConfig {
    type Target = RpcServiceConfig;

    fn deref(&self) -> &Self::Target {
        &self.rpc
    }
}

impl_logging_config!(RpcFrameConfig);

#[peace_config]
pub struct RpcServiceConfig {
    /// The address and port the `gRPC` server listens on.
    #[default("127.0.0.1:50051".parse().unwrap())]
    #[arg(short = 'H', long, default_value = "127.0.0.1:50051")]
    pub rpc_addr: SocketAddr,

    /// Using unix domain socket instead of TCP/IP socket.
    /// Only for unix systems.
    ///
    /// If configured, `uds` will be preferred over `addr`.
    #[arg(long)]
    pub rpc_uds: Option<PathBuf>,

    /// Enable admin rpc service.
    #[cfg(feature = "admin_endpoints")]
    #[default(false)]
    #[arg(short = 'A', long)]
    pub rpc_admin_endpoints: bool,

    /// Admin rpc service `Authorization` `Bearer` token.
    #[arg(short = 'T', long)]
    pub rpc_admin_token: Option<String>,

    /// Enable reflection service.
    #[cfg(feature = "reflection")]
    #[default(false)]
    #[arg(short = 'R', long)]
    pub rpc_reflection: bool,

    /// Allow this server to accept http1 requests.
    ///
    /// Accepting http1 requests is only useful when developing grpc-web enabled services.
    /// If this setting is set to true but services are not correctly configured to handle grpc-web requests,
    /// your server may return confusing (but correct) protocol errors.
    ///
    /// Default is false.
    #[default(false)]
    #[arg(short, long)]
    pub rpc_accept_http1: bool,

    /// Set the concurrency limit applied to on requests inbound per connection.
    #[arg(long)]
    pub rpc_concurrency_limit_per_connection: Option<usize>,

    /// Sets whether to use an adaptive flow control.
    /// Defaults to false.
    /// Enabling this will override the limits set in http2_initial_stream_window_size and http2_initial_connection_window_size.
    #[default(Some(false))]
    #[arg(long)]
    pub rpc_http2_adaptive_window: Option<bool>,

    /// Set whether HTTP2 Ping frames are enabled on accepted connections.
    ///
    /// If None is specified, HTTP2 keepalive is disabled,
    /// otherwise the duration specified will be the time interval between HTTP2 Ping frames.
    /// The timeout for receiving an acknowledgement of the keepalive ping can be set with Server::http2_keepalive_timeout.
    ///
    /// Default is no HTTP2 keepalive (None)
    #[arg(long)]
    pub rpc_http2_keepalive_interval: Option<u64>,

    /// Sets a timeout for receiving an acknowledgement of the keepalive ping.
    ///
    /// If the ping is not acknowledged within the timeout, the connection will be closed. Does nothing if http2_keep_alive_interval is disabled.
    ///
    /// Default is 20 seconds.
    #[default(Some(20))]
    #[arg(long, default_value = "20")]
    pub rpc_http2_keepalive_timeout: Option<u64>,

    /// Sets the max connection-level flow control for HTTP2
    ///
    /// Default is 65,535
    #[default(Some(65535))]
    #[arg(long, default_value = "65535")]
    pub rpc_initial_connection_window_size: Option<u32>,

    /// Sets the SETTINGS_INITIAL_WINDOW_SIZE option for HTTP2 stream-level flow control.
    ///
    /// Default is 65,535
    #[default(Some(65535))]
    #[arg(long, default_value = "65535")]
    pub rpc_initial_stream_window_size: Option<u32>,

    /// Sets the SETTINGS_MAX_CONCURRENT_STREAMS option for HTTP2 connections.
    ///
    /// Default is no limit (None).
    #[arg(long)]
    pub rpc_max_concurrent_streams: Option<u32>,

    /// Sets the maximum frame size to use for HTTP2.
    ///
    /// Passing None will do nothing.
    ///
    /// If not set, will default from underlying transport.
    #[arg(long)]
    pub rpc_max_frame_size: Option<u32>,

    /// Set a timeout on for all request handlers.
    #[arg(long)]
    pub rpc_req_timeout: Option<u64>,

    /// TLS configurations.
    #[clap(flatten)]
    pub rpc_tls_config: TlsConfig,

    /// Set the value of TCP_NODELAY option for accepted connections.
    ///
    /// Enabled by default.
    #[default(true)]
    #[arg(long, default_value = "true")]
    pub rpc_tcp_nodelay: bool,

    /// Set how often to send TCP keepalive probes.
    /// By default TCP keepalive probes is disabled.
    #[arg(long)]
    pub rpc_tcp_keepalive: Option<u64>,
}
