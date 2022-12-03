use clap::Parser;
use clap_serde_derive::ClapSerde;
use peace_logs::LogLevel;
use std::default::Default;
use std::{net::SocketAddr, path::PathBuf};

/// Base Command Line Interface (CLI) for Peace-RPC framework.
#[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
#[command(name = "peace-rpc", author, version, about, propagate_version = true)]
pub struct RpcFrameConfig {
    /// The address and port the `gRPC` server listens on.
    #[default("127.0.0.1:50051".parse().unwrap())]
    #[arg(short = 'H', long, default_value = "127.0.0.1:50051")]
    pub addr: SocketAddr,

    /// Logging level.
    #[default(LogLevel::Info)]
    #[arg(short = 'L', long, value_enum, default_value = "info")]
    pub log_level: LogLevel,

    /// Logging env filter.
    #[arg(short = 'F', long, value_enum)]
    pub log_env_filter: Option<String>,

    /// Turning on debug will display information such as code line number, source file, thread id, etc.
    #[default(false)]
    #[arg(short, long)]
    pub debug: bool,

    /// Enable admin rpc service.
    #[cfg(feature = "admin_rpc")]
    #[default(false)]
    #[arg(short = 'A', long)]
    pub admin_rpc: bool,

    /// Enable reflection service.
    #[cfg(feature = "reflection")]
    #[default(false)]
    #[arg(short = 'R', long)]
    pub reflection: bool,

    /// Allow this server to accept http1 requests.
    ///
    /// Accepting http1 requests is only useful when developing grpc-web enabled services.
    /// If this setting is set to true but services are not correctly configured to handle grpc-web requests,
    /// your server may return confusing (but correct) protocol errors.
    ///
    /// Default is false.
    #[default(false)]
    #[arg(short, long)]
    pub accept_http1: bool,

    /// Set the concurrency limit applied to on requests inbound per connection.
    #[arg(long)]
    pub concurrency_limit_per_connection: Option<usize>,

    /// Sets whether to use an adaptive flow control.
    /// Defaults to false.
    /// Enabling this will override the limits set in http2_initial_stream_window_size and http2_initial_connection_window_size.
    #[default(Some(false))]
    #[arg(long)]
    pub http2_adaptive_window: Option<bool>,

    /// Set whether HTTP2 Ping frames are enabled on accepted connections.
    ///
    /// If None is specified, HTTP2 keepalive is disabled,
    /// otherwise the duration specified will be the time interval between HTTP2 Ping frames.
    /// The timeout for receiving an acknowledgement of the keepalive ping can be set with Server::http2_keepalive_timeout.
    ///
    /// Default is no HTTP2 keepalive (None)
    #[arg(long)]
    pub http2_keepalive_interval: Option<u64>,

    /// Sets a timeout for receiving an acknowledgement of the keepalive ping.
    ///
    /// If the ping is not acknowledged within the timeout, the connection will be closed. Does nothing if http2_keep_alive_interval is disabled.
    ///
    /// Default is 20 seconds.
    #[default(Some(20))]
    #[arg(long, default_value = "20")]
    pub http2_keepalive_timeout: Option<u64>,

    /// Sets the max connection-level flow control for HTTP2
    ///
    /// Default is 65,535
    #[default(Some(65535))]
    #[arg(long, default_value = "65535")]
    pub initial_connection_window_size: Option<u32>,

    /// Sets the SETTINGS_INITIAL_WINDOW_SIZE option for HTTP2 stream-level flow control.
    ///
    /// Default is 65,535
    #[default(Some(65535))]
    #[arg(long, default_value = "65535")]
    pub initial_stream_window_size: Option<u32>,

    /// Sets the SETTINGS_MAX_CONCURRENT_STREAMS option for HTTP2 connections.
    ///
    /// Default is no limit (None).
    #[arg(long)]
    pub max_concurrent_streams: Option<u32>,

    /// Sets the maximum frame size to use for HTTP2.
    ///
    /// Passing None will do nothing.
    ///
    /// If not set, will default from underlying transport.
    #[arg(long)]
    pub max_frame_size: Option<u32>,

    /// Set a timeout on for all request handlers.
    #[arg(long)]
    pub req_timeout: Option<u64>,

    /// Enabled `tls` support.
    #[default(false)]
    #[arg(short, long)]
    pub tls: bool,

    /// SSL certificate path.
    #[arg(short = 'C', long)]
    pub ssl_cert: Option<PathBuf>,

    /// SSL certificate key path.
    #[arg(short = 'K', long)]
    pub ssl_key: Option<PathBuf>,

    /// Set the value of TCP_NODELAY option for accepted connections.
    ///
    /// Enabled by default.
    #[default(true)]
    #[arg(long, default_value = "true")]
    pub tcp_nodelay: bool,

    /// Set how often to send TCP keepalive probes.
    /// By default TCP keepalive probes is disabled.
    #[arg(long)]
    pub tcp_keepalive: Option<u64>,
}

crate::impl_config!(RpcFrameConfig);
impl_logger_config!(RpcFrameConfig);
