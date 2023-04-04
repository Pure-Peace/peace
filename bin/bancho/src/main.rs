#[allow(unused_imports)]
#[macro_use]
extern crate peace_rpc;

#[macro_use]
extern crate peace_logs;

pub mod app;
pub mod rpc;

pub use app::*;
pub use rpc::*;

pub async fn run(
    cfg: std::sync::Arc<BanchoConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new instance of the `App.
    let app = App::new(cfg);

    // Initialize the logger with the frame configuration of the `App`.
    peace_logs::init(&app.cfg.frame_cfg);

    // Start serving the RPC server with the `App` instance.
    peace_rpc::server::serve(app).await;

    Ok(())
}

/// The main entry point of the application.
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = BanchoConfig::get();
    peace_runtime::runtime(&cfg.runtime_cfg).unwrap().block_on(run(cfg))
}
