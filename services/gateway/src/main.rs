use peace_api::components;
use peace_gateway::{cmd::PeaceGatewayArgs, App};

#[tokio::main]
pub async fn main() {
    let app = App::new(PeaceGatewayArgs::get());
    peace_logs::init_with_args(&app.args.api_framework_args);

    components::http::serve(app).await;
}