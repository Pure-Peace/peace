use peace_api::http;
use peace_gateway::{App, GatewayConfig};

#[tokio::main]
pub async fn main() {
    let app = App::new(GatewayConfig::get());
    peace_logs::init(&app.cfg.frame_cfg);

    http::serve(app).await;
}
