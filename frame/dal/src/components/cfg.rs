use clap::Parser;
use clap_serde_derive::ClapSerde;
use peace_logs::LogLevel;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::{net::SocketAddr, path::PathBuf};

/// Base Command Line Interface (CLI) for Peace-dal framework.
#[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
#[command(name = "peace-dal", author, version, about, propagate_version = true)]
pub struct DbConfig {}

crate::impl_config!(DbConfig);
