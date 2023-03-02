#[allow(unused_imports)]
#[macro_use]
extern crate peace_logs;

#[macro_use]
extern crate peace_api;

pub mod app;

pub use app::*;

#[tokio::main(flavor = "multi_thread")]
pub async fn main() {
    // Create a new instance of the `App.
    let app = App::new(BanchoStandaloneConfig::get());

    // Initialize the logger with the frame configuration of the `App`.
    peace_logs::init(&app.cfg.frame_cfg);

    // Start serving the HTTP(s) server with the `App` instance.
    peace_api::http::serve(app).await;
}
