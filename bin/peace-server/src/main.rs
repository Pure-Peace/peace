use peace_bancho::objects::{Bancho, Peace};
use peace_database::Database;
use peace_settings::local::LocalConfig;

use ntex::web::types::Data;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    // Create local settings
    let cfg = LocalConfig::init();

    // Create database object includes postgres and redis pool
    let database = Data::new(
        Database::new(
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
