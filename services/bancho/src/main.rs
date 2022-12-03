use bancho::{cfg::BanchoConfig, App};
use peace_rpc::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(BanchoConfig::get());
    peace_logs::init(&app.cfg.frame_cfg);

    server::serve(app).await;

    Ok(())
}