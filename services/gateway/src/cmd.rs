use clap::Parser;
use once_cell::sync::OnceCell;
use peace_api::components::cmd::PeaceApiArgs;
use std::sync::Arc;

/// Command Line Interface (CLI) for Peace gateway service.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "peace-gateway",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct PeaceGatewayArgs {
    /// A list of hostnames to route to the bancho service.
    #[arg(short = 'B', long)]
    pub bancho_hostname: Vec<String>,

    #[command(flatten)]
    pub api_framework_args: PeaceApiArgs,
}

impl PeaceGatewayArgs {
    /// Get or init [`PeaceGatewayArgs`]
    pub fn get() -> Arc<PeaceGatewayArgs> {
        static ARGS: OnceCell<Arc<PeaceGatewayArgs>> = OnceCell::new();
        ARGS.get_or_init(|| Arc::new(PeaceGatewayArgs::parse())).clone()
    }
}
