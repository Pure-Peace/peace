use peace_api::http;
use peace_gateway::{App, GatewayConfig};

#[tokio::main(flavor = "multi_thread")]
pub async fn main() {
    let app = App::new(GatewayConfig::get());
    peace_logs::init(&app.cfg.frame_cfg);

    http::serve(app).await;
}
