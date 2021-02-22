use async_std::sync::RwLock;
use colored::Colorize;

use peace::constants::PEACE_BANNER;
use peace::database::Database;
use peace::objects::PlayerSessions;
use peace::settings::{model::Settings, peace::start_server};

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
    start_server(cfg, settings, database, player_sessions).await
}
