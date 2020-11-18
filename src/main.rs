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
mod utils;

use std::sync::Arc;

use async_std::sync::RwLock;
use colored::Colorize;

use constants::types::{PlayerMap, TestType};
use database::Database;
use settings::{types::Settings, BANNER};

use dashmap::DashMap;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print banner
    println!("{}", BANNER.green());

    // Create Peace's settings
    let (cfg, settings) = Settings::new().unwrap();

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;
    let data: TestType = Arc::new(RwLock::new(0));
    let player_map: PlayerMap = Arc::new(DashMap::with_capacity(100));
    // Start actix server
    settings::actix::start_server(cfg, database, data, player_map).await
}
