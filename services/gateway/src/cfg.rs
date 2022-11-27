use clap::Parser;
use clap_serde_derive::ClapSerde;
use peace_api::{cfg::ApiFrameConfig, impl_config};
use serde::{Deserialize, Serialize};

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
    /// A list of hostnames to route to the bancho service.
    #[arg(short = 'B', long)]
    pub bancho_hostname: Vec<String>,

    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,
}

impl_config!(GatewayConfig);
