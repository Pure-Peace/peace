#![allow(dead_code)]
extern crate config;
extern crate derivative;
extern crate serde;

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

use actix_web::web::Data;
use objects::{Bancho, Peace};

use crate::database::Database;
use crate::settings::model::LocalConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create local settings
    let local_config = LocalConfig::init();

    // Create database object includes postgres and redis pool
    let database = Data::new(Database::new(&local_config).await);

    // Create bancho object
    let bancho = Data::new(Bancho::init(&local_config, &database).await);

    // Create and start
    let mut peace = Peace::new(bancho.clone(), database);

    peace.start().await
}
