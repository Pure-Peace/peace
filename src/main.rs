#![allow(dead_code)]
//#![allow(unused_imports)]

#[macro_use]
extern crate log;
extern crate config;
extern crate serde;

mod constants;
mod database;
mod handlers;
mod packets;
mod routes;
mod settings;
mod utils;

use colored::Colorize;
use database::Database;
use settings::{types::Settings, BANNER};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print banner
    println!("{}", BANNER.green());

    // Create Peace's settings
    let (cfg, settings) = Settings::new().unwrap();

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;

    // Start actix server
    settings::actix::start_server(cfg, database).await
}
