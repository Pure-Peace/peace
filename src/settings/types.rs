use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub postgres: deadpool_postgres::Config,
    pub redis: deadpool_redis::Config,

    pub check_pools_on_created: bool,
    pub env: String,
    pub debug: bool,
    pub server: Server,
    hello: Hello,
}

#[derive(Debug, Deserialize)]
pub struct Hello {
    world: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub token: String,
    pub secret: String,
}
