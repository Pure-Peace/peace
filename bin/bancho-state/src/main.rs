#[macro_use]
extern crate peace_rpc;

#[macro_use]
extern crate peace_logs;

pub mod app;
pub mod rpc;

pub use app::*;
pub use rpc::*;

/// The main entry point of the application.
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new instance of the `App` struct with the configuration loaded from the file system.
    let app = App::new(BanchoStateConfig::get());

    // Initialize the logger with the frame configuration of the `App`.
    peace_logs::init(&app.cfg.frame_cfg);

    // Start serving the RPC server with the `App` instance.
    peace_rpc::server::serve(app).await;

    Ok(())
}
