#[macro_use]
extern crate peace_rpc;

#[allow(unused_imports)]
#[macro_use]
extern crate peace_logs;

pub mod app;
pub mod rpc;

pub use app::*;
pub use rpc::*;

use peace_services::DumpConfig;

pub async fn run(
    cfg: std::sync::Arc<ChatServiceConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new instance of the `App.
    let app = App::initialize(cfg.clone()).await;

    // Start serving the RPC server with the `App` instance.
    peace_rpc::server::serve(app.clone()).await;

    if cfg.chat_service_dump_configs.save_dump() {
        let _ = app
            .chat_service
            .try_dump_to_disk(
                cfg.chat_service_dump_configs.dump_type(),
                cfg.chat_service_dump_configs.dump_path(),
            )
            .await;
    }

    Ok(())
}

/// The main entry point of the application.
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tools::main_startup_info!();

    let cfg = ChatServiceConfig::get();
    // Initialize the logger.
    peace_logs::init(&cfg.frame_cfg);

    // Initialize runtime and run app.
    peace_runtime::runtime(&cfg.runtime_cfg).unwrap().block_on(run(cfg))
}
