pub mod connectors;
use connectors::*;

use colored::Colorize;

#[cfg(feature = "with_peace")]
use peace_constants::{DB_VERSION, PEACE_VERSION};

/// Database object
///
/// Includes postgres and redis deadpool
#[derive(Clone)]
pub struct Database {
    pub pg: Postgres,
    pub redis: Redis,
}

impl Database {
    pub async fn new(
        postgres_cfg: &deadpool_postgres::Config,
        redis_cfg: &deadpool_redis::Config,
        #[cfg(feature = "with_peace")] check_db_version: bool,
        check_connect: bool,
    ) -> Self {
        println!(
            "> {}",
            "Initializing database connections...".bright_purple()
        );
        let pg = Postgres::new(postgres_cfg, check_connect).await;
        let redis = Redis::new(redis_cfg, check_connect).await;
        println!(
            "> {}",
            "Database connection initialization successfully!\n"
                .bold()
                .bright_purple()
        );
        let database = Database { pg, redis };
        #[cfg(feature = "with_peace")]
        if check_db_version {
            database.check_version().await;
        }

        database
    }

    #[cfg(feature = "with_peace")]
    pub async fn check_version(&self) {
        println!("> {}", "Checking Database version...".bright_purple());
        match self
            .pg
            .query_first(
                r#"SELECT * FROM "public"."versions" WHERE "version" = $1;"#,
                &[&PEACE_VERSION],
            )
            .await
        {
            Ok(row) => {
                let db_version: &str = row.get("db_version");
                if db_version != DB_VERSION {
                    println!(
                        "> {}",
                        format!("Inconsistent database versions, there may be updates or lags?")
                            .bold()
                            .yellow()
                    );
                }
                println!(
                    "> {}",
                    format!(
                        "Peace version: {}; Target db version: {}; Current db version: {}",
                        PEACE_VERSION, db_version, DB_VERSION
                    )
                    .bold()
                    .yellow()
                );
                println!(
                    "> {}",
                    "Database initialization success!\n".bold().bright_purple()
                );
            }
            Err(err) => {
                println!(
                    "> {}",
                    format!(
                        "Failed to check database version! Error: {}; Peace version: {}; Current db version: {}\n",
                        err, PEACE_VERSION, DB_VERSION
                    )
                    .yellow()
                    .red()
                );
            }
        };
    }
}
