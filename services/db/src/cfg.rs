use clap::Parser;
use clap_serde_derive::ClapSerde;
use serde::{Deserialize, Serialize};

use peace_rpc::{cfg::RpcFrameConfig, impl_config};

/// Command Line Interface (CLI) for DB service.
#[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
#[command(name = "db", author, version, about, propagate_version = true)]
pub struct DbConfig {
    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,
}

impl_config!(DbConfig);
