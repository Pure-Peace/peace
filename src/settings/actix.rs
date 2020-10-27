use crate::database::Database;
use crate::routes;

use actix_web::{dev::Server, middleware::Logger, App, HttpServer};
use config::Config;
use std::time::Instant;

use colored::Colorize;


/// Actix before start
pub async fn before_start(cfg: &Config) -> (String, String) {
    // Load cfg
    let addr: String = cfg.get("addr").unwrap_or("127.0.0.1:8080".to_string());
    let log_format: String = cfg.get("logger.actix_log_format").unwrap_or_else(|error| {
        error!(
            "Failed to get config key: {}, use default value. Raw error: {}",
            "logger.actix_log_format".red(),
            error
        );
        r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T" "%{Referer}i""#.to_string()
    });

    (addr, log_format)
}

/// Actix started
pub async fn started(cfg: &Config, addr: &str) -> Instant {
    // Server started
    let text = format!("Service is Running at http://{}", addr).green();
    info!("{}", text);
    Instant::now()
}

/// Actix stopped
pub async fn stopped(server: Server, start_time: Instant) -> std::io::Result<()> {
    // Waiting for server stopped
    let err = server.await;
    let title = format!("Service has Stopped!").yellow();
    let time_string = format!("Service running time: {:?}\n", start_time.elapsed()).bold().bright_blue();
    info!("{} \n\n {}", title, time_string);
    err
}

/// Run actix
pub async fn start_server(cfg: &Config, database: Database) -> std::io::Result<()> {
    let (addr, log_format): (String, String) = before_start(&cfg).await;
    // Run server
    info!("{}", "Starting http service...".bold().bright_blue());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(&log_format))
            .data(database.clone())
            .configure(routes::init)
    })
    .bind(&addr)
    .unwrap()
    .run();
    // Wait for stopped
    let start_time = started(&cfg, &addr).await;
    stopped(server, start_time).await
}
