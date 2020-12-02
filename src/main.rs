#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate config;
extern crate serde;

mod constants;
mod database;
mod handlers;
mod objects;
mod packets;
mod routes;
mod settings;
mod types;
mod utils;

use async_std::sync::RwLock;
use colored::Colorize;

use database::Database;
use settings::{objects::Settings, BANNER};
use types::TestType;

use objects::PlayerSessions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print banner
    println!("{}", BANNER.green());

    // Create Peace's settings
    let (cfg, settings) = Settings::new().unwrap();

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;

    let data: TestType = RwLock::new(0);

    // Create PlayerSession for this server
    let player_sessions = RwLock::new(PlayerSessions::new(100, database.clone()));

    // Start actix server
    settings::actix::start_server(cfg, database, data, player_sessions).await
}
