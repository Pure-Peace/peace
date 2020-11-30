pub mod connectors;

use crate::settings::objects::Settings;
use colored::Colorize;

use connectors::*;

/// Database object
///
/// Includes postgres and redis deadpool
#[derive(Clone)]
pub struct Database {
    pub pg: Postgres,
    pub redis: Redis,
}

impl Database {
    pub async fn new(settings: &Settings) -> Self {
        println!("> {}", "Initializing database...".bright_purple());
        let pg = Postgres::new(settings).await;
        let redis = Redis::new(settings).await;
        println!("> {}", "Database initialization success!\n".bold().bright_purple());
        Database { pg, redis }
    }
}
