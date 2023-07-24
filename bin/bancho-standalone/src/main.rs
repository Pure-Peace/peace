#[allow(unused_imports)]
#[macro_use]
extern crate peace_logs;

#[macro_use]
extern crate peace_api;

pub mod app;

pub use app::*;

pub async fn run(
    cfg: std::sync::Arc<BanchoStandaloneConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new instance of the `App.
    let app = App::initialize(cfg.clone()).await;

    // Start serving the HTTP(s) server with the `App` instance.
    peace_api::http::serve(app.clone()).await;

    if cfg.chat_service_dump_configs.chat_save_dump {
        app.chat_service
            .try_dump_to_disk(&cfg.chat_service_dump_configs.chat_dump_path)
            .await?
    }

    Ok(())
}

/// The main entry point of the application.
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tools::main_startup_info!();

    let cfg = BanchoStandaloneConfig::get();
    // Initialize the logger.
    peace_logs::init(&cfg.frame_cfg);

    // Initialize runtime and run app.
    peace_runtime::runtime(&cfg.runtime_cfg).unwrap().block_on(run(cfg))
}
