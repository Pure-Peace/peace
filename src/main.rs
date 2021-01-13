#![allow(dead_code)]
#![allow(unused_imports)]

extern crate config;
extern crate serde;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use async_std::sync::RwLock;
use colored::Colorize;

mod constants;
mod database;
mod events;
mod handlers;
mod objects;
mod packets;
mod renders;
mod routes;
mod settings;
mod types;
mod utils;

use constants::PEACE_BANNER;
use database::Database;
use objects::PlayerSessions;
use settings::model::Settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print banner
    println!("{}", PEACE_BANNER.green());

    // Create Peace's settings
    let (cfg, settings) = Settings::new().unwrap();

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;

    // Create PlayerSession for this server
    let player_sessions = RwLock::new(PlayerSessions::new(100, database.clone()));

    // Start peace server
    settings::peace::start_server(cfg, settings, database, player_sessions).await
}
