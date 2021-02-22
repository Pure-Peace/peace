use peace::constants::PEACE_PP_SERVER_BANNER;
use peace::database::Database;
use peace::settings::{model::Settings, pp_server::start_server};

use colored::Colorize;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print banner
    println!("{}", PEACE_PP_SERVER_BANNER.green());

    // Create Peace's settings
    let (cfg, settings) = Settings::new()
        .expect("Settings failed to initialize, please check the local configuration file.");

    // Create database object includes postgres and redis pool
    let database = Database::new(&settings).await;

    // Start pp server
    start_server(cfg, settings, database).await
}
