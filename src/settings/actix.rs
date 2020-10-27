use crate::database::Database;
use crate::routes;

use actix_web::{dev::Server, middleware::Logger, App, HttpServer};
use config::Config;
use std::time::Instant;

/// Actix before start
pub async fn before_start(cfg: Config) {}

/// Actix started
pub async fn started(cfg: Config, addr: &str) -> Instant {
    // Server started
    info!("Peace: Running at http://{}", addr);
    Instant::now()
}

/// Actix stopped
pub async fn stopped(server: Server, start_time: Instant) -> std::io::Result<()> {
    // Waiting for server stopped
    let err = server.await;
    info!(
        "Peace: Stopped! \n\n Service running time: {:?}\n",
        start_time.elapsed()
    );
    err
}

/// Run actix
pub async fn start_server(cfg: Config, database: Database) -> std::io::Result<()> {
    // Start server
    let addr: &str = cfg.get("addr").unwrap_or("127.0.0.1:8080");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(database.clone())
            .configure(routes::init)
    })
    .bind(&addr)
    .unwrap()
    .run();

    let start_time = started(cfg.clone(), addr).await;
    stopped(server, start_time).await
}
