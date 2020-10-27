use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub postgres: deadpool_postgres::Config,
    pub redis: deadpool_redis::Config,

    pub check_pools_on_created: bool,
    pub env: String,
    pub debug: bool,
    pub server: Server,
    pub logger: Logger,
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

#[derive(Debug, Deserialize)]
pub struct LoggerMode {
    debug: String,
    error: String,
    warn: String,
    info: String,
}

#[derive(Debug, Deserialize)]
pub struct Logger {
    pub level: String,
    pub mode: LoggerMode,
}
