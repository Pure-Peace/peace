pub use crate::macro_impl_config as impl_config;

#[cfg(feature = "derive")]
pub use peace_cfg_derive::*;

use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use clap_serde_derive::ClapSerde;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    ops::Deref,
    path::{Path, PathBuf},
    process,
};

#[derive(Parser)]
pub struct BaseConfig<T>
where
    T: Parser + ClapSerde + Args + serde::Serialize,
{
    #[clap(flatten)]
    pub config_path: ConfigPath,

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
    /// Start service.
    Run,
    /// Create default configuration file.
    CreateConfig(ConfigPath),
}

#[derive(Args, ClapSerde, Debug, Clone, Serialize, Deserialize)]

pub struct TlsConfig {
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
}

#[derive(Args)]
pub struct ConfigPath {
    /// Configuration file path (Support `.yml`, `.json`, `toml`).
    #[arg(short = 'c', long = "config", default_value = "config.json")]
    pub path: PathBuf,
}

impl Deref for ConfigPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.path.as_path()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFileType {
    Yaml,
    Json,
    Toml,
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
            "toml" => Self::Toml,
            _ => panic!("Unsupported config file type \".{}\"", ext),
        }
    }
}

macro_rules! cfg_to_string {
    ($file_type: expr, $cfg: expr, $(($typ: ident, $serde: ident, $fn: ident)),*) => {
        match $file_type {
            $(ConfigFileType::$typ => $serde::$fn($cfg).unwrap(),)*
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
                (Yaml, serde_yaml, to_string),
                (Json, serde_json, to_string_pretty),
                (Toml, toml, to_string_pretty)
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
    File::open(f.path).map(|mut file| match f.ext_type {
        ConfigFileType::Yaml =>
            serde_yaml::from_reader::<_, <T as ClapSerde>::Opt>(file).unwrap(),
        ConfigFileType::Json =>
            serde_json::from_reader::<_, <T as ClapSerde>::Opt>(file).unwrap(),
        ConfigFileType::Toml => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            toml::from_slice(&buf).unwrap()
        },
    })
}

pub trait ParseConfig<T> {
    fn parse_cfg() -> T;
}

impl<T> ParseConfig<T> for T
where
    T: Parser + ClapSerde + Args + serde::Serialize,
{
    /// Parses args from the command line,
    /// performs a `command` or returns a loaded app configuration.
    fn parse_cfg() -> T {
        let cfg = BaseConfig::<T>::parse();
        let f = ConfigFile::new(&cfg.config_path);
        let cfg_t =
            read_config_from_file::<T>(&f).map(T::from).unwrap_or(cfg.config);

        if let Some(Commands::CreateConfig(path)) = cfg.command {
            let f = ConfigFile::new(&path);
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
}

#[async_trait]
pub trait RpcClientConfig {
    type RpcClient;
    fn uri(&self) -> &str;
    fn uds(&self) -> Option<&std::path::PathBuf>;
    fn tls(&self) -> bool;
    fn ssl_cert(&self) -> Option<&std::path::PathBuf>;
    fn lazy_connect(&self) -> bool;
    async fn connect_client(&self) -> Result<Self::RpcClient, anyhow::Error>;
}

pub mod macros {
    pub use anyhow::Error;
    pub use async_trait::async_trait;
    pub use once_cell::sync::OnceCell;
    pub use paste;
    pub use peace_logs;

    /// Add a `get` method to the given struct.
    ///
    /// This method only initializes the configuration struct when it is called
    /// for the first time, and subsequent calls will directly return the
    /// [`std::sync::Arc<T>`] of the struct.
    ///
    /// ```rust
    /// use clap::Parser;
    /// use clap_serde_derive::ClapSerde;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
    /// pub struct GatewayConfig {
    ///     /// Bancho service address.
    ///     #[default("http://127.0.0.1:50051".to_owned())]
    ///     #[arg(long, default_value = "http://127.0.0.1:50051")]
    ///     pub bancho_addr: String,
    /// }
    ///
    /// peace_cfg::impl_config!(GatewayConfig);
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
                    use $crate::ParseConfig;
                    static CFG: $crate::macros::OnceCell<std::sync::Arc<$t>> =
                        $crate::macros::OnceCell::<std::sync::Arc<$t>>::new();
                    CFG.get_or_init(|| std::sync::Arc::new(<$t>::parse_cfg()))
                        .clone()
                }
            }
        };
    }

    #[macro_export]
    /// Define RPC config struct
    ///
    /// Example
    /// ```rust,ignore
    /// #[macro_use]
    /// extern crate peace_api;
    ///
    /// use clap_serde_derive::ClapSerde;
    /// use peace_api::{ApiFrameConfig, Application, RpcClientConfig};
    ///
    /// // Should import RPC service namespace first
    /// use peace_pb::services::bancho_rpc;
    ///
    /// // It will generate a config struct named [`BanchoRpcConfig`]
    /// peace_api::define_rpc_client_config!(
    ///     service_name: bancho_rpc,
    ///     config_name: BanchoRpcConfig
    /// );
    ///
    /// /// Command Line Interface (CLI) for Peace gateway service.
    /// #[peace_config]
    /// #[command(
    ///     name = "peace-gateway",
    ///     author,
    ///     version,
    ///     about,
    ///     propagate_version = true
    /// )]
    /// pub struct GatewayConfig {
    ///     #[command(flatten)]
    ///     pub frame_cfg: ApiFrameConfig,
    ///
    ///     #[command(flatten)]
    ///     pub bancho: BanchoRpcConfig, // Use it - [`BanchoRpcConfig`]
    /// }
    ///
    /// // Init config
    /// let cfg = GatewayConfig::get();
    ///
    /// // Connect to RPC client
    /// let client = cfg.bancho.connect_client().await.unwrap_or_else(|err| {
    ///     error!("Unable to connect to the bancho gRPC service, please make sure the service is started.");
    ///     panic!("{}", err)
    /// });
    /// ```
    macro_rules! macro_define_rpc_client_config {
        (service_name: $service_name: ty, config_name: $config_name: ty) => {
            $crate::macro_define_rpc_client_config!(service_name: $service_name, config_name: $config_name, default_uri: "http://127.0.0.1:50051");
        };
        (service_name: $service_name: ty, config_name: $config_name: ty, default_uri: $default_uri: literal) => {
            $crate::macros::paste::paste! {
                pub type [<$service_name:camel>] =
                    [<$service_name:snake>]::[<$service_name:snake _client>]::[<$service_name:camel Client>]<tonic::transport::Channel>;

                #[derive(
                    clap::Parser, clap_serde_derive::ClapSerde, Debug, Clone, serde::Serialize, serde::Deserialize,
                )]
                pub struct $config_name {
                    /// Service uri.
                    #[default($default_uri.to_owned())]
                    #[arg(long, default_value = $default_uri)]
                    pub [<$service_name:snake _uri>]: String,

                    /// Service unix domain socket path.
                    /// Only for unix systems.
                    ///
                    /// `uds` will be preferred over `uri`.
                    #[arg(long)]
                    pub [<$service_name:snake _uds>]: Option<std::path::PathBuf>,

                    /// Enable `tls` connection.
                    #[default(false)]
                    #[arg(long)]
                    pub [<$service_name:snake _tls>]: bool,

                    /// SSL certificate path.
                    #[arg(long)]
                    pub [<$service_name:snake _ssl_cert>]: Option<std::path::PathBuf>,

                    /// Not attempt to connect to the endpoint until first use.
                    #[default(false)]
                    #[arg(long)]
                    pub [<$service_name:snake _lazy_connect>]: bool,
                }

                #[$crate::macros::async_trait]
                impl $crate::RpcClientConfig for $config_name {
                    type RpcClient = [<$service_name:camel>];

                    #[inline]
                    fn uri(&self) -> &str {
                        self.[<$service_name:snake _uri>].as_str()
                    }

                    #[inline]
                    fn uds(&self) -> Option<&std::path::PathBuf> {
                        self.[<$service_name:snake _uds>].as_ref()
                    }

                    #[inline]
                    fn tls(&self) -> bool {
                        self.[<$service_name:snake _tls>]
                    }

                    #[inline]
                    fn ssl_cert(&self) -> Option<&std::path::PathBuf> {
                        self.[<$service_name:snake _ssl_cert>].as_ref()
                    }

                    #[inline]
                    fn lazy_connect(&self) -> bool {
                        self.[<$service_name:snake _lazy_connect>]
                    }

                    #[inline]
                    async fn connect_client(&self) -> Result<Self::RpcClient, $crate::macros::Error> {
                        async fn connect_endpoint(
                            endpoint: tonic::transport::Endpoint,
                            lazy_connect: bool,
                        ) -> Result<tonic::transport::Channel, $crate::macros::Error> {
                            Ok(if lazy_connect {
                                endpoint.connect_lazy()
                            } else {
                                $crate::macros::peace_logs::info!(concat!("Attempting to connect to ", stringify!($service_name), " gRPC endpoint..."));
                                endpoint.connect().await?
                            })
                        }

                        #[cfg(unix)]
                        if let Some(uds) = self.uds().to_owned() {
                            $crate::macros::peace_logs::info!(concat!(stringify!($service_name), " gRPC service: {}"), uds);
                            let service_factory =
                                tower::service_fn(|_| tokio::net::UnixStream::connect(uds));
                            let endpoint =
                                tonic::transport::Endpoint::try_from("http://[::]:50051")?;

                            let channel = if self.lazy_connect() {
                                endpoint.connect_with_connector_lazy(service_factory)
                            } else {
                                $crate::macros::peace_logs::info!(concat!("Attempting to connect to ", stringify!($service_name), " gRPC endpoint..."));
                                endpoint.connect_with_connector(service_factory).await?
                            };
                            return Self::RpcClient::new(channel);
                        }

                        $crate::macros::peace_logs::info!(concat!(stringify!($service_name), " gRPC service: {}"), self.uri());
                        if self.tls() {
                            let pem =
                                tokio::fs::read(self.ssl_cert().as_ref().unwrap())
                                    .await?;
                            let ca = tonic::transport::Certificate::from_pem(pem);
                            let tls = tonic::transport::ClientTlsConfig::new().ca_certificate(ca);
                            return Ok(Self::RpcClient::new(
                                connect_endpoint(
                                    tonic::transport::Channel::from_shared(self.uri().to_owned())?
                                        .tls_config(tls)?,
                                    self.lazy_connect(),
                                )
                                .await?,
                            ));
                        }

                        Ok(Self::RpcClient::new(
                            connect_endpoint(
                                tonic::transport::Channel::from_shared(self.uri().to_owned())?,
                                self.lazy_connect(),
                            )
                            .await?,
                        ))
                    }
                }
            }
        };
    }
}
