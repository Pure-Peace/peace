#![allow(dead_code)]
extern crate config;
extern crate serde;
extern crate derivative;

#[macro_use]
extern crate log;

pub mod constants;
pub mod database;
pub mod events;
pub mod handlers;
pub mod objects;
pub mod packets;
pub mod renders;
pub mod routes;
pub mod settings;
pub mod types;
pub mod utils;


use async_std::sync::RwLock;
use colored::Colorize;

use crate::constants::PEACE_BANNER;
use crate::database::Database;
use crate::objects::PlayerSessions;
use crate::settings::{model::Settings, peace};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print banner
    println!("{}", PEACE_BANNER.green());

    // Create Peace's settings
    let (cfg, settings) = Settings::new()
        .expect("Settings failed to initialize, please check the local configuration file.");

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;

    // Create PlayerSession for this server
    let player_sessions = RwLock::new(PlayerSessions::new(100, database.clone()));

    // Start peace server
    peace::start_server(cfg, settings, database, player_sessions).await
}
