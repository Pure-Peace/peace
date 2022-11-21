#[macro_use]
extern crate peace_logs;

pub mod cmd;
pub mod components;
pub mod routes;

use axum::Server;

use clap::Parser;

#[tokio::main]
pub async fn main() {
    let args = cmd::PeaceGatewayArgs::parse();
    peace_logs::init_with_args(&args);

    let app = components::app(&args);

    // Start server
    let addr = &args.listen.parse().unwrap();
    info!("{}", tools::listening!(addr));
    Server::bind(addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(components::shutdown_signal())
        .await
        .unwrap();
}
