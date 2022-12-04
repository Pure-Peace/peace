pub use crate::macro_impl_config as impl_config;

use clap::{Args, Parser, Subcommand};
use clap_serde_derive::ClapSerde;
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
        ConfigFileType::Yaml => {
            serde_yaml::from_reader::<_, <T as ClapSerde>::Opt>(file).unwrap()
        },
        ConfigFileType::Json => {
            serde_json::from_reader::<_, <T as ClapSerde>::Opt>(file).unwrap()
        },
        ConfigFileType::Toml => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            toml::from_slice(&buf).unwrap()
        },
    })
}

pub trait ParseConfig<T> {
    fn parse() -> T;
}

impl<T> ParseConfig<T> for T
where
    T: Parser + ClapSerde + Args + serde::Serialize,
{
    /// Parses args from the command line,
    /// performs a `command` or returns a loaded app configuration.
    fn parse() -> T {
        let cfg = BaseConfig::<T>::parse();
        let f = ConfigFile::new(&cfg.config_path);
        let cfg_t = read_config_from_file::<T>(&f)
            .map(|c| T::from(c))
            .unwrap_or(cfg.config);

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
                    static CFG: $crate::macros::OnceCell<std::sync::Arc<$t>> =
                        $crate::macros::OnceCell::<std::sync::Arc<$t>>::new();
                    CFG.get_or_init(|| std::sync::Arc::new(<$t>::parse()))
                        .clone()
                }
            }
        };
    }
}
