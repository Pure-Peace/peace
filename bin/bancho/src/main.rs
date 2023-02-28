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
    let app = App::new(BanchoConfig::get());
    peace_logs::init(&app.cfg.frame_cfg);

    server::serve(app).await;

    Ok(())
}
