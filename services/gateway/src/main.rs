use peace_gateway::components;

#[tokio::main]
pub async fn main() {
    let args = components::cmd::PeaceGatewayArgs::get();
    peace_logs::init_with_args(args);

    components::http::serve(args).await
}
