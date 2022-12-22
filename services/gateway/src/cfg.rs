use clap::Parser;
use clap_serde_derive::ClapSerde;
use peace_api::{cfg::ApiFrameConfig, impl_config};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Command Line Interface (CLI) for Peace gateway service.
#[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
#[command(
    name = "peace-gateway",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct GatewayConfig {
    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    /// A list of hostnames to route to the bancho service.
    #[arg(short = 'B', long)]
    pub bancho_hostname: Vec<String>,

    /// Bancho service address.
    #[default("http://127.0.0.1:50051".to_owned())]
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    pub bancho_addr: String,

    /// Bancho service unix domain socket path.
    /// Only for unix systems.
    ///
    /// `uds` will be preferred over `addr`.
    #[arg(long)]
    pub bancho_uds: Option<PathBuf>,

    /// `tls` connection for Bancho service.
    #[default(false)]
    #[arg(long)]
    pub bancho_tls: bool,

    /// SSL certificate path.
    #[arg(long)]
    pub bancho_ssl_cert: Option<PathBuf>,
}

impl_config!(GatewayConfig);
