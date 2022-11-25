use clap::Parser;
use once_cell::sync::OnceCell;
use std::{net::SocketAddr, path::PathBuf};

/// Command Line Interface (CLI) for Peace-Gateway service.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "peace-gateway",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct PeaceGatewayArgs {
    /// The address and port the `http` server listens on.
    #[arg(short = 'H', long, default_value = "127.0.0.1:8000")]
    pub http_addr: SocketAddr,

    /// The address and port the `https` server listens on.
    #[cfg(feature = "tls")]
    #[arg(short = 'S', long, default_value = "127.0.0.1:443")]
    pub https_addr: SocketAddr,

    /// Logging level.
    #[arg(short = 'L', long, value_enum, default_value = "info")]
    pub log_level: peace_logs::LogLevel,

    /// Logging env filter.
    #[arg(short = 'F', long, value_enum)]
    pub log_env_filter: Option<String>,

    /// Turning on debug will display information such as code line number, source file, thread id, etc.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    /// Enabled admin api.
    #[arg(short = 'A', long, default_value_t = true)]
    pub admin_api: bool,

    /// Admin api `Authorization` `bearer` token.
    #[arg(short, long)]
    pub admin_token: Option<String>,

    /// Limit the max number of in-flight requests.
    #[arg(short, long, default_value_t = 1024)]
    pub concurrency_limit: usize,

    /// Fail requests that take longer than timeout (secs).
    #[arg(short, long, default_value_t = 10)]
    pub req_timeout: u64,

    /// Enabled `hostname-based` routing.
    #[arg(short = 'N', long, default_value_t = false)]
    pub hostname_routing: bool,

    /// A list of hostnames to route to the bancho service.
    #[arg(short = 'B', long)]
    pub bancho_hostname: Vec<String>,

    /// Enabled `tls` support.
    #[cfg(feature = "tls")]
    #[arg(short, long, default_value_t = false)]
    pub tls: bool,

    /// SSL certificate path.
    #[cfg(feature = "tls")]
    #[arg(short = 'C', long)]
    pub ssl_cert: Option<PathBuf>,

    /// SSL certificate key path.
    #[cfg(feature = "tls")]
    #[arg(short = 'K', long)]
    pub ssl_key: Option<PathBuf>,

    /// Redirect http to https.
    #[cfg(feature = "tls")]
    #[arg(short, long, default_value_t = false)]
    pub force_https: bool,

    /// Set the value of TCP_NODELAY option for accepted connections.
    #[arg(long, default_value_t = false)]
    pub tcp_nodelay: bool,

    /// Set whether to sleep on accept errors, to avoid exhausting file descriptor limits.
    #[arg(long, default_value_t = true)]
    pub tcp_sleep_on_accept_errors: bool,

    /// Set how often to send TCP keepalive probes.
    /// By default TCP keepalive probes is disabled.
    #[arg(long)]
    pub tcp_keepalive: Option<u64>,

    /// Set the duration between two successive TCP keepalive retransmissions,
    /// if acknowledgement to the previous keepalive transmission is not received.
    #[arg(long)]
    pub tcp_keepalive_interval: Option<u64>,

    /// Set the number of retransmissions to be carried out before declaring that remote end is not available.
    #[arg(long)]
    pub tcp_keepalive_retries: Option<u32>,

    /// The `swagger ui` base uri path.
    #[arg(long, default_value = "/swagger-ui/")]
    pub swagger_path: String,

    /// The `openapi.json` uri path.
    #[arg(long, default_value = "/api-doc/openapi.json")]
    pub openapi_json: String,
}

impl PeaceGatewayArgs {
    /// Get or init [`PeaceGatewayArgs`]
    pub fn get() -> &'static Self {
        static ARGS: OnceCell<PeaceGatewayArgs> = OnceCell::new();
        ARGS.get_or_init(|| PeaceGatewayArgs::parse())
    }
}

impl peace_logs::LoggerArgs for PeaceGatewayArgs {
    fn log_level(&self) -> peace_logs::LogLevel {
        self.log_level
    }

    fn env_filter(&self) -> Option<String> {
        self.log_env_filter.clone()
    }

    fn debug(&self) -> bool {
        self.debug
    }
}
