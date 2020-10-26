#![allow(dead_code)]
//#![allow(unused_imports)]

extern crate config;
extern crate serde;

mod database;
mod handlers;
mod routes;
mod settings;

use database::Database;
use settings::types::Settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create Peace's settings
    let (cfg, settings) = Settings::new().unwrap();

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;

    // Start actix server
    settings::actix::start_server(cfg, database).await
}
