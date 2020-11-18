#![allow(dead_code)]
//#![allow(unused_imports)]

#[macro_use]
extern crate log;
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
use settings::{types::Settings, BANNER};
use types::{PlayerSessions, TestType};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print banner
    println!("{}", BANNER.green());

    // Create Peace's settings
    let (cfg, settings) = Settings::new().unwrap();

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;
    let data: TestType = RwLock::new(0);
    let player_sessions_map: PlayerSessions = RwLock::new(hashbrown::HashMap::with_capacity(100));
    // Start actix server
    settings::actix::start_server(cfg, database, data, player_sessions_map).await
}
