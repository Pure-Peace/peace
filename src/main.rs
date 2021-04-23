extern crate derivative;
extern crate serde;

#[macro_use]
extern crate log;

pub mod events;
pub mod handlers;
pub mod objects;
pub mod renders;
pub mod routes;
pub mod types;

use actix_web::web::Data;
use objects::{Bancho, Peace};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create local settings
    let cfg = peace_settings::local::LocalConfig::init();

    // Create database object includes postgres and redis pool
    let database = Data::new(
        peace_database::Database::new(
            &cfg.data.postgres,
            &cfg.data.redis,
            cfg.data.check_db_version_on_created,
            cfg.data.check_pools_on_created,
        )
        .await,
    );

    // Create bancho object
    let bancho = Data::new(Bancho::init(&cfg, &database).await);

    // Create and start
    let mut peace = Peace::new(bancho.clone(), database);

    peace.start().await
}
