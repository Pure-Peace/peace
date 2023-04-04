#[macro_use]
extern crate peace_logs;

#[macro_use]
extern crate peace_api;

pub mod app;

pub use app::*;

pub async fn run(cfg: std::sync::Arc<GatewayConfig>) {
    // Create a new instance of the `App.
    let app = App::new(cfg);

    // Start serving the HTTP(s) server with the `App` instance.
    peace_api::http::serve(app).await;
}

/// The main entry point of the application.
pub fn main() {
    let cfg = GatewayConfig::get();
    // Initialize the logger.
    peace_logs::init(&cfg.frame_cfg);

    // Initialize runtime and run app.
    peace_runtime::runtime(&cfg.runtime_cfg).unwrap().block_on(run(cfg))
}
