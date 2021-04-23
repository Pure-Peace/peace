use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct LocalConfigData {
    pub postgres: deadpool_postgres::Config,
    pub redis: deadpool_redis::Config,
    pub check_pools_on_created: bool,
    pub check_db_version_on_created: bool,

    pub env: String,
    pub debug: bool,
    pub server: Server,
    pub pp_server: PpServer,
    pub geoip: Geoip,
    pub logger: Logger,
    #[serde(rename = "prometheus")]
    pub prom: Prometheus,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub data_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PpServer {
    pub url: String,
    pub pp_calc_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggerMode {
    debug: String,
    error: String,
    warn: String,
    info: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Logger {
    pub level: String,
    pub mode: LoggerMode,
    pub actix_log_format: String,
    pub exclude_endpoints: Vec<String>,
    pub exclude_endpoints_regex: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Geoip {
    pub enabled: bool,
    pub mmdb_path: String,
    pub web_api: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Prometheus {
    pub namespace: String,
    pub endpoint: String,
    pub exclude_endpoint_log: bool,
}
