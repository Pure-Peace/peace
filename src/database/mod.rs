pub mod connectors;

use crate::settings::types::Settings;

use connectors::*;

/// Database object
/// 
/// Includes postgres and redis deadpool
#[derive(Clone)]
pub struct Database {
    pub pg: Postgres,
    pub redis: Redis
}

impl Database {
    pub async fn new(settings: &Settings) -> Self {
        let pg = Postgres::new(settings).await;
        let redis = Redis::new(settings).await;
        Database {
            pg,
            redis
        }
    }
}


