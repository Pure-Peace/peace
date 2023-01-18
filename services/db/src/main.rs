use peace_db::{App, DbServiceConfig};
use peace_rpc::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(DbServiceConfig::get());
    peace_logs::init(&app.cfg.frame_cfg);

    server::serve(app).await;

    Ok(())
}
