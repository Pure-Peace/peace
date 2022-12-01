use clap::{Args, Parser, Subcommand};
use clap_serde_derive::ClapSerde;
use once_cell::sync::OnceCell;
use peace_logs::{LogLevel, LoggerConfig};
use std::default::Default;
use std::io::Write;
use std::path::Path;
use std::process;
use std::{fs::File, net::SocketAddr, path::PathBuf, sync::Arc};

#[derive(Parser)]
pub struct BaseConfig<T>
where
    T: Parser + ClapSerde + Args + serde::Serialize,
{
    /// Using configuration file.
    #[arg(short = 'c', long = "config", default_value = "config.yml")]
    pub config_path: PathBuf,

    /// Save the current parameters as a configuration file.
    #[arg(long, default_value = "false")]
    pub save_as_config: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Rest of arguments
    #[clap(flatten)]
    pub config: T,
}

#[derive(Subcommand)]
enum Commands {
    /// Start gateway service.
    Run,
    /// Create default configuration file.
    CreateConfig(CreateConfig),
}

/// Base Command Line Interface (CLI) for Peace-api framework.
#[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
#[command(name = "peace-api", author, version, about, propagate_version = true)]
pub struct ApiFrameConfig {
    /// The address and port the `http` server listens on.
    #[default("127.0.0.1:8000".parse().unwrap())]
    #[arg(short = 'H', long, default_value = "127.0.0.1:8000")]
    pub http_addr: SocketAddr,

    /// The address and port the `https` server listens on.
    #[cfg(feature = "tls")]
    #[default("127.0.0.1:443".parse().unwrap())]
    #[arg(short = 'S', long, default_value = "127.0.0.1:443")]
    pub https_addr: SocketAddr,

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

    /// Enabled admin api.
    #[default(true)]
    #[arg(short = 'A', long)]
    pub admin_api: bool,

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

    /// Enabled `tls` support.
    #[cfg(feature = "tls")]
    #[default(false)]
    #[arg(short, long)]
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
    #[default(false)]
    #[arg(short, long)]
    pub force_https: bool,

    /// Set the value of TCP_NODELAY option for accepted connections.
    #[default(false)]
    #[arg(long)]
    pub tcp_nodelay: bool,

    /// Set whether to sleep on accept errors, to avoid exhausting file descriptor limits.
    #[default(true)]
    #[arg(long)]
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

#[derive(Args)]
struct CreateConfig {
    /// Configuration file path.
    #[arg(short = 'c', long = "config", default_value = "config.yml")]
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFileType {
    Yaml,
    Json,
}

#[derive(Debug, Clone)]
pub struct ConfigFile<'a> {
    pub path: &'a Path,
    pub ext_type: ConfigFileType,
}

impl<'a> ConfigFile<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path, ext_type: ConfigFileType::from(path) }
    }
}

impl ApiFrameConfig {
    /// Get or init [`ApiFrameConfig`]
    pub fn get() -> Arc<ApiFrameConfig> {
        static CFG: OnceCell<Arc<ApiFrameConfig>> = OnceCell::new();
        CFG.get_or_init(|| Arc::new(ApiFrameConfig::parse())).clone()
    }
}

impl LoggerConfig for ApiFrameConfig {
    fn log_level(&self) -> LogLevel {
        self.log_level
    }

    fn env_filter(&self) -> Option<String> {
        self.log_env_filter.clone()
    }

    fn debug(&self) -> bool {
        self.debug
    }
}

impl<P> From<P> for ConfigFileType
where
    P: AsRef<Path>,
{
    fn from(path: P) -> Self {
        let binding = path.as_ref().extension().unwrap().to_ascii_lowercase();
        let ext = binding.to_str().unwrap();
        match ext {
            "yml" => Self::Yaml,
            "json" => Self::Json,
            _ => panic!("Unsupported config file type \"{}\"", ext),
        }
    }
}

macro_rules! cfg_from_reader {
    ($file_type: expr, $file: expr, $(($typ: ident, $reader: ident)),*) => {
        match $file_type {
            $(ConfigFileType::$typ => $reader::from_reader::<_, <T as ClapSerde>::Opt>($file).expect("Error in configuration file"),)*
        }
    };
}

macro_rules! cfg_to_string {
    ($file_type: expr, $cfg: expr, $(($typ: ident, $serde: ident)),*) => {
        match $file_type {
            $(ConfigFileType::$typ => $serde::to_string($cfg).unwrap(),)*
        }
    };
}

pub fn write_config<T>(f: &ConfigFile, content: T)
where
    T: serde::Serialize,
{
    File::create(f.path)
        .unwrap()
        .write_all(
            cfg_to_string!(
                f.ext_type,
                &content,
                (Yaml, serde_yaml),
                (Json, serde_json)
            )
            .as_bytes(),
        )
        .unwrap();
}

pub fn read_config_from_file<T>(
    f: &ConfigFile,
) -> Result<<T as ClapSerde>::Opt, std::io::Error>
where
    T: ClapSerde,
{
    File::open(f.path).map(|file| {
        cfg_from_reader!(
            f.ext_type,
            file,
            (Yaml, serde_yaml),
            (Json, serde_json)
        )
    })
}


/// Parses args from the command line,
/// performs a `command` or returns a loaded app configuration.
pub fn parse<T>() -> T
where
    T: Parser + ClapSerde + Args + serde::Serialize,
{
    let cfg = BaseConfig::<T>::parse();
    let f = ConfigFile::new(cfg.config_path.as_path());
    let cfg_t = read_config_from_file::<T>(&f)
        .map(|c| T::from(c))
        .unwrap_or(cfg.config);

    if let Some(Commands::CreateConfig(create)) = cfg.command {
        let f = ConfigFile::new(&create.config_path);
        write_config(&f, &cfg_t);
        println!(
            "[OK] Configuration files have been written to: `{}`",
            f.path.to_str().unwrap()
        );
        process::exit(0)
    }

    if cfg.save_as_config {
        write_config(&f, &cfg_t);
    }

    cfg_t
}

pub mod macros {
    pub use once_cell::sync::OnceCell;

    /// Implement a `get` function for the given struct that can be used to get app configuration.
    ///
    /// ```rust
    /// use clap::Parser;
    /// use clap_serde_derive::ClapSerde;
    /// use peace_api::{cfg::ApiFrameConfig, impl_config};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
    /// pub struct GatewayConfig {
    ///     /// A list of hostnames to route to the bancho service.
    ///     #[arg(short = 'B', long)]
    ///     pub bancho_hostname: Vec<String>,
    ///
    ///     #[command(flatten)]
    ///     pub frame_cfg: ApiFrameConfig,
    /// }
    ///
    /// impl_config!(GatewayConfig);
    ///
    /// fn main() {
    ///     GatewayConfig::get();
    /// }
    /// ```
    #[macro_export]
    macro_rules! macro_impl_config {
        ($t: ty) => {
            impl $t {
                /// Get or init configs (only initialized once).
                pub fn get() -> std::sync::Arc<$t> {
                    static CFG: $crate::cfg::macros::OnceCell<std::sync::Arc<$t>> =
                        $crate::cfg::macros::OnceCell::<std::sync::Arc<$t>>::new();
                    CFG.get_or_init(
                        || std::sync::Arc::new($crate::cfg::parse()),
                    )
                    .clone()
                }
            }
        };
    }
}
