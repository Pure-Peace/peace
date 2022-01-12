use ntex::web::types::Data;

use peace_database::Database;
use peace_pp_server::objects::{glob::Glob, PPserver};
use peace_pp_server::settings::LocalConfig;

#[ntex::main]
async fn main() {
    // Create local settings
    let cfg = LocalConfig::init();

    #[cfg(feature = "with_peace")]
    // Create database object includes postgres and redis pool
    let database = Database::new(
        &cfg.data.postgres,
        &cfg.data.redis,
        cfg.data.check_db_version_on_created,
        cfg.data.check_pools_on_created,
    )
    .await;

    // Create Glob object
    let glob = Data::new(
        Glob::init(
            &cfg,
            #[cfg(feature = "with_peace")]
            &database,
        )
        .await,
    );

    // Create and start
    let mut server = PPserver::new(glob.clone());

    let _err = server.start().await;
}
