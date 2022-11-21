use clap::Parser;

/// Command Line Interface (CLI) for Peace-Gateway service.
#[derive(Parser, Debug)]
#[command(
    name = "peace-gateway",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct PeaceGatewayArgs {
    /// The address and port the server listens on.
    #[arg(short, long, default_value = "127.0.0.1:8000")]
    pub listen: String,
    /// Logging level.
    #[arg(short = 'L', long, value_enum, default_value = "info")]
    pub log_level: peace_logs::LogLevel,
    /// Debug mode (extremely verbose logs).
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
    /// Admin api `Authorization` `bearer` token.
    #[arg(short, long)]
    pub admin_token: Option<String>,
    /// Limit the max number of in-flight requests.
    #[arg(short, long, default_value_t = 1024)]
    pub concurrency_limit: usize,
    /// Fail requests that take longer than timeout (secs).
    #[arg(short, long, default_value_t = 10)]
    pub req_timeout: u64,
}

impl peace_logs::LoggerArgs for PeaceGatewayArgs {
    fn log_level(&self) -> peace_logs::LogLevel {
        self.log_level
    }

    fn debug(&self) -> bool {
        self.debug
    }
}
