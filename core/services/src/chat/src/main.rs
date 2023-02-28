use chat::{App, ChatServiceConfig};
use peace_rpc::server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(ChatServiceConfig::get());
    peace_logs::init(&app.cfg.frame_cfg);

    server::serve(app).await;

    Ok(())
}
