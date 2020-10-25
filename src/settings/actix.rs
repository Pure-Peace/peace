use crate::database::Database;
use crate::handlers::bancho::*;

use actix_web::{dev::Server, App, HttpServer};
use config::Config;
use std::time::Instant;

/// Actix before start
pub async fn before_start(cfg: Config) {}

/// Actix started
pub async fn started(cfg: Config, addr: &str) -> Instant {
    // Server started
    println!("Peace: Running at http://{}", addr);
    Instant::now()
}

/// Actix stopped
pub async fn stopped(server: Server, start_time: Instant) -> std::io::Result<()> {
    // Waiting for server stopped
    let err = server.await;
    println!(
        "Peace: Stopped! \n\n Service running time: {:?}\n",
        start_time.elapsed()
    );
    err
}

/// Run actix
pub async fn start_server(cfg: Config, database: Database) -> std::io::Result<()> {
    // Start server
    let addr: &str = cfg.get("addr").unwrap_or("127.0.0.1:8080");
    let server = HttpServer::new(move || App::new().data(database.clone()).service(index))
        .bind(&addr)
        .unwrap()
        .run();

    let start_time = started(cfg.clone(), addr).await;
    stopped(server, start_time).await
}
