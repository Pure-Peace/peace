pub use crate::macro_impl_config as impl_config;

#[cfg(feature = "derive")]
pub use peace_cfg_derive::*;

use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use clap_serde_derive::ClapSerde;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    ops::Deref,
    path::{Path, PathBuf},
    process,
};

/// A generic configuration struct that holds a configuration file path,
/// a boolean flag to save the current parameters as a configuration file,
/// a command, and any additional arguments provided.
#[derive(Parser)]
pub struct BaseConfig<T>
where
    T: Parser + ClapSerde + Args + serde::Serialize,
{
    /// Configuration file path.
    #[clap(flatten)]
    pub config_path: ConfigPath,

    /// Save the current parameters as a configuration file.
    #[arg(long, default_value = "false")]
    pub save_as_config: bool,

    /// Command to execute.
    #[command(subcommand)]
    command: Option<Commands>,

    /// Additional arguments.
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

/// An enum representing supported configuration file types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFileType {
    Yaml,
    Json,
    Toml,
}

/// A struct representing a configuration file path and its associated file
/// type.
#[derive(Debug, Clone)]
pub struct ConfigFile<'a> {
    /// The path of the configuration file.
    pub path: &'a Path,
    /// The file type of the configuration file.
    pub ext_type: ConfigFileType,
}

impl<'a> ConfigFile<'a> {
    /// Creates a new [`ConfigFile`] instance from the provided file path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the file path.
    ///
    /// # Returns
    ///
    /// A new [`ConfigFile`] instance.
    pub fn new(path: &'a Path) -> Self {
        Self { path, ext_type: ConfigFileType::from(path) }
    }
}

impl<P> From<P> for ConfigFileType
where
    P: AsRef<Path>,
{
    /// Converts a file path into a [`ConfigFileType`] instance based on the
    /// file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - A file path to convert.
    ///
    /// # Returns
    ///
    /// A [`ConfigFileType`] instance representing the file type.
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

/// This macro is used to serialize a given configuration to a string using one
/// of several possible file formats, depending on the specified configuration
/// file type.
///
/// # Arguments
///
/// * `$file_type` - The file type to use for serialization.
/// * `$cfg` - The configuration to serialize.
/// * `$(($typ: ident, $serde: ident, $fn: ident)),*` - A list of tuples
///   representing each possible
/// file format to use for serialization. Each tuple contains an identifier for
/// the file format, an identifier for the corresponding serde library, and an
/// identifier for the serde function to use for serialization.
macro_rules! cfg_to_string {
    ($file_type: expr, $cfg: expr, $(($typ: ident, $serde: ident, $fn: ident)),*) => {
        match $file_type {
            $(ConfigFileType::$typ => $serde::$fn($cfg).unwrap(),)*
        }
    };
}

/// Writes a configuration file of the given content to the specified path.
///
/// # Arguments
///
/// * `f` - A [`ConfigFile`] containing the path and file extension for the
///   configuration file.
/// * `content` - The configuration file contents to be written.
///
/// # Panics
///
/// Panics if the file cannot be created or if the contents cannot be written to
/// the file.
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

/// Reads a configuration file of the specified type from the specified path.
///
/// # Arguments
///
/// * `f` - A [`ConfigFile`] containing the path and file extension for the
///   configuration file.
///
/// # Returns
///
/// Returns the deserialized configuration data.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the file cannot be read or the contents
/// cannot be deserialized.
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
/// Trait for parsing configuration for a type.
pub trait ParseConfig<T> {
    /// Parses args from the command line,
    /// performs a `command` or returns a loaded app configuration.
    fn parse_cfg() -> T;
}

impl<T> ParseConfig<T> for T
where
    T: Parser + ClapSerde + Args + serde::Serialize,
{
    /// Parses args from the command line,
    /// performs a `command` or returns a loaded app configuration.
    fn parse_cfg() -> T {
        // Parse the base configuration from command line arguments.
        let cfg = BaseConfig::<T>::parse();
        // Create a new configuration file based on the parsed configuration
        // path.
        let f = ConfigFile::new(&cfg.config_path);
        // Attempt to read the configuration file and deserialize it into the
        // target type `T`. If there is an error reading the file, use the
        // default configuration from the base configuration instead.
        let cfg_t =
            read_config_from_file::<T>(&f).map(T::from).unwrap_or(cfg.config);

        // If the command is to create a new configuration file, write the
        // current configuration to the specified file path and exit the
        // program.
        if let Some(Commands::CreateConfig(path)) = cfg.command {
            let f = ConfigFile::new(&path);
            write_config(&f, &cfg_t);
            println!(
                "[OK] Configuration files have been written to: `{}`",
                f.path.to_str().unwrap()
            );
            process::exit(0)
        }

        // If the save_as_config option is specified, write the current
        // configuration to the configuration file.
        if cfg.save_as_config {
            write_config(&f, &cfg_t);
        }

        // Return the final configuration.
        cfg_t
    }
}

#[async_trait]
pub trait RpcClientConfig {
    /// The type of the RPC client that will be created
    type RpcClient;

    /// Gets the URI for the RPC client to connect to
    fn uri(&self) -> &str;

    /// Gets the UDS (Unix domain socket) path, if one is specified
    fn uds(&self) -> Option<&std::path::PathBuf>;

    /// Determines whether TLS is enabled for the RPC client
    fn tls(&self) -> bool;

    /// Gets the path to the SSL certificate, if TLS is enabled
    fn ssl_cert(&self) -> Option<&std::path::PathBuf>;

    /// Determines whether to lazily connect the RPC client
    fn lazy_connect(&self) -> bool;

    /// Connects the RPC client
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Error` if the client could not be connected.
    async fn connect_client(&self) -> Result<Self::RpcClient, anyhow::Error>;
}

#[async_trait]
pub trait DbConfig {
    /// Returns a configured [`ConnectOptions`]
    fn configured_opt(&self) -> ConnectOptions;

    /// Connect to database.
    async fn connect(&self) -> Result<DatabaseConnection, DbErr> {
        Database::connect(self.configured_opt()).await
    }
}

pub mod macros {
    pub mod ____private {
        pub use anyhow::Error;
        pub use async_trait::async_trait;
        pub use clap;
        pub use clap_serde_derive::ClapSerde;
        pub use once_cell::sync::OnceCell;
        pub use paste;
        pub use peace_logs::{self, log::LevelFilter, LogLevel};
        pub use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
        pub use serde::{Deserialize, Serialize};
    }

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
                    static CFG: $crate::macros::____private::OnceCell<
                        std::sync::Arc<$t>,
                    > = $crate::macros::____private::OnceCell::<
                        std::sync::Arc<$t>,
                    >::new();
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
            $crate::macros::____private::paste::paste! {
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

                #[$crate::macros::____private::async_trait]
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
                    async fn connect_client(&self) -> Result<Self::RpcClient, $crate::macros::____private::Error> {
                        async fn connect_endpoint(
                            endpoint: tonic::transport::Endpoint,
                            lazy_connect: bool,
                        ) -> Result<tonic::transport::Channel, $crate::macros::____private::Error> {
                            Ok(if lazy_connect {
                                endpoint.connect_lazy()
                            } else {
                                $crate::macros::____private::peace_logs::info!(concat!("Attempting to connect to ", stringify!($service_name), " gRPC endpoint..."));
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

                        $crate::macros::____private::peace_logs::info!(concat!(stringify!($service_name), " gRPC service: {}"), self.uri());
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

    /// Create a configuration structure for the specified database.
    ///
    /// [clap #3513](https://github.com/clap-rs/clap/issues/3513)
    /// Due to `clap` currently does not support adding a prefix to the
    /// configuration struct, this declarative macro is currently used to reuse
    /// configuration items.
    ///
    /// examples
    ///
    /// ```rust
    /// use peace_cfg::define_db_config;
    ///
    /// define_db_config!(config_name: BanchoDbConfig, command_prefix: bancho);
    /// define_db_config!(config_name: LazerDbConfig, command_prefix: lazer);
    /// ```
    #[macro_export]
    macro_rules! define_db_config {
        (config_name: $struct_name: ident, command_prefix: $prefix: ident) => {
            $crate::macros::____private::paste::paste! {
                #[allow(non_snake_case)]
                mod [<__def_ $struct_name _mod__>] {
                    use $crate::macros::____private::{clap, ClapSerde, Deserialize, Serialize};

                    /// Database configurations
                    #[derive(
                        clap::Parser, clap_serde_derive::ClapSerde, Debug, Clone, serde::Serialize, serde::Deserialize,
                    )]
                    pub struct $struct_name {
                        /// Database connection URL.
                        #[default("protocol://username:password@host/database".to_string())]
                        #[arg(
                            long,
                            default_value = "protocol://username:password@host/database"
                        )]
                        pub [<$prefix _db_url>]: String,

                        /// Set the maximum number of connections of the pool.
                        #[arg(long)]
                        pub [<$prefix _db_max_connections>]: Option<u32>,

                        /// Set the minimum number of connections of the pool.
                        #[arg(long)]
                        pub [<$prefix _db_min_connections>]: Option<u32>,

                        /// Set the timeout duration when acquiring a connection.
                        #[arg(long)]
                        pub [<$prefix _db_connect_timeout>]: Option<u64>,

                        /// Set the maximum amount of time to spend waiting for acquiring a connection.
                        #[arg(long)]
                        pub [<$prefix _db_acquire_timeout>]: Option<u64>,

                        /// Set the idle duration before closing a connection.
                        #[arg(long)]
                        pub [<$prefix _db_idle_timeout>]: Option<u64>,

                        /// Set the maximum lifetime of individual connections.
                        #[arg(long)]
                        pub [<$prefix _db_max_lifetime>]: Option<u64>,

                        /// Enable SQLx statement logging (default true).
                        #[default(true)]
                        #[arg(long, default_value = "true")]
                        pub [<$prefix _db_sqlx_logging>]: bool,

                        /// Set SQLx statement logging level (default [`LogLevel::Info`]) (ignored if `sqlx_logging` is false).
                        #[default($crate::macros::____private::LogLevel::Info)]
                        #[arg(long, value_enum, default_value = "info")]
                        pub [<$prefix _db_sqlx_logging_level>]: $crate::macros::____private::LogLevel,

                        /// Set schema search path (PostgreSQL only).
                        #[arg(long)]
                        pub [<$prefix _db_set_schema_search_path>]: Option<String>,
                    }

                    $crate::impl_db_config!($struct_name, $prefix);
                }

                pub use [<__def_ $struct_name _mod__>]::$struct_name;
            }
        };
    }

    #[macro_export]
    macro_rules! impl_db_config {
        ($struct_name: ident, $prefix: ident) => {
            $crate::macros::____private::paste::paste! {
                impl $crate::DbConfig for $struct_name {
                    fn configured_opt(&self) -> $crate::macros::____private::ConnectOptions {
                        let mut opt = $crate::macros::____private::ConnectOptions::new(self.[<$prefix _db_url>].clone());

                        if let Some(v) = self.[<$prefix _db_max_connections>] {
                            opt.max_connections(v);
                        }
                        if let Some(v) = self.[<$prefix _db_min_connections>] {
                            opt.min_connections(v);
                        }
                        if let Some(v) =
                            self.[<$prefix _db_connect_timeout>].map(std::time::Duration::from_secs)
                        {
                            opt.connect_timeout(v);
                        }
                        if let Some(v) =
                            self.[<$prefix _db_acquire_timeout>].map(std::time::Duration::from_secs)
                        {
                            opt.acquire_timeout(v);
                        }
                        if let Some(v) =
                            self.[<$prefix _db_idle_timeout>].map(std::time::Duration::from_secs)
                        {
                            opt.idle_timeout(v);
                        }
                        if let Some(v) =
                            self.[<$prefix _db_max_lifetime>].map(std::time::Duration::from_secs)
                        {
                            opt.max_lifetime(v);
                        }
                        if let Some(v) = self.[<$prefix _db_set_schema_search_path>].to_owned() {
                            opt.set_schema_search_path(v);
                        }
                        opt.sqlx_logging(self.[<$prefix _db_sqlx_logging>]);
                        opt.sqlx_logging_level($crate::macros::____private::LevelFilter::from(
                            self.[<$prefix _db_sqlx_logging_level>],
                        ));

                        opt
                    }
                }
            }
        };
    }
}
