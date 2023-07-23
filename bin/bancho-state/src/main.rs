#[allow(unused_imports)]
#[macro_use]
extern crate peace_logs;

#[macro_use]
extern crate peace_rpc;

pub mod app;
pub mod rpc;

pub use app::*;
pub use rpc::*;

pub async fn run(
    cfg: std::sync::Arc<BanchoStateConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new instance of the `App.
    let app = App::initialize(cfg).await;

    // Start serving the RPC server with the `App` instance.
    peace_rpc::server::serve(app).await;

    Ok(())
}

/// The main entry point of the application.
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tools::main_startup_info!();

    let cfg = BanchoStateConfig::get();
    // Initialize the logger.
    peace_logs::init(&cfg.frame_cfg);

    // Initialize runtime and run app.
    peace_runtime::runtime(&cfg.runtime_cfg).unwrap().block_on(run(cfg))
}
