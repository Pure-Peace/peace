use clap_serde_derive::ClapSerde;
use peace_cfg::TlsConfig;
use peace_cfg::{impl_config, peace_config, SingletonConfig};
use peace_logs::LoggingConfigArgs;
use std::{default::Default, net::SocketAddr, ops::Deref};

/// Basic configuration items for `peace-api` framework.
#[peace_config]
pub struct ApiFrameConfig {
    /// Logging configurations.
    #[clap(flatten)]
    pub logging: LoggingConfigArgs,

    #[clap(flatten)]
    pub api: ApiServiceConfig,
}

impl Deref for ApiFrameConfig {
    type Target = ApiServiceConfig;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl_logging_config!(ApiFrameConfig);

#[peace_config]
pub struct ApiServiceConfig {
    /// The address and port the `http` server listens on.
    #[default("127.0.0.1:8000".parse().unwrap())]
    #[arg(short = 'H', long, default_value = "127.0.0.1:8000")]
    pub http_addr: SocketAddr,

    /// The address and port the `https` server listens on.
    #[default("127.0.0.1:443".parse().unwrap())]
    #[arg(short = 'S', long, default_value = "127.0.0.1:443")]
    pub https_addr: SocketAddr,

    /// TLS configurations.
    #[clap(flatten)]
    pub tls_config: TlsConfig,

    /// Enabled admin api.
    #[default(false)]
    #[arg(short = 'A', long)]
    pub admin_endpoints: bool,

    /// Admin api `Authorization` `bearer` token.
    #[arg(short, long)]
    pub admin_token: Option<String>,

    /// Limit the max number of in-flight requests.
    #[default(1024)]
    #[arg(long, default_value = "1024")]
    pub concurrency_limit: usize,

    /// Fail requests that take longer than timeout (secs).
    #[default(10)]
    #[arg(short, long, default_value = "10")]
    pub req_timeout: u64,

    /// Enabled `hostname-based` routing.
    #[default(false)]
    #[arg(short = 'N', long)]
    pub hostname_routing: bool,

    /// Redirect http to https.
    #[default(false)]
    #[arg(short, long)]
    pub force_https: bool,

    /// Set the value of TCP_NODELAY option for accepted connections.
    #[default(false)]
    #[arg(long)]
    pub tcp_nodelay: bool,

    /// Set whether to sleep on accept errors, to avoid exhausting file descriptor limits.
    #[default(true)]
    #[arg(long, default_value = "true")]
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
    #[arg(long, default_value = "/swagger-ui")]
    pub swagger_path: String,

    /// The `openapi.json` uri path.
    #[arg(long, default_value = "/api-doc/openapi.json")]
    pub openapi_json: String,
}
