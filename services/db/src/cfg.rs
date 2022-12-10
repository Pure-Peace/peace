use clap::Parser;
use clap_serde_derive::ClapSerde;
use peace_dal::db::peace::PeaceDbConfig;
use peace_rpc::{cfg::RpcFrameConfig, impl_config};
use serde::{Deserialize, Serialize};

/// Command Line Interface (CLI) for DB service.
#[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
#[command(name = "db", author, version, about, propagate_version = true)]
pub struct DbServiceConfig {
    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,

    #[command(flatten)]
    pub peace_db: PeaceDbConfig,
}

impl_config!(DbServiceConfig);
