use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct LocalConfigData {
    #[cfg(feature = "with_peace")]
    pub postgres: deadpool_postgres::Config,
    #[cfg(feature = "with_peace")]
    pub redis: deadpool_redis::Config,
    #[cfg(not(feature = "with_peace"))]
    pub osu_api_keys: Vec<String>,

    #[cfg(feature = "with_peace")]
    pub check_pools_on_created: bool,
    #[cfg(feature = "with_peace")]
    pub check_db_version_on_created: bool,
    #[cfg(feature = "with_peace")]
    pub peace_key: String,
    #[cfg(feature = "with_peace")]
    pub peace_url: String,

    pub env: String,
    pub debug: bool,
    pub osu_files_dir: String,
    pub recalculate_osu_file_md5: bool,
    pub preload_osu_files: bool,
    pub beatmap_cache_max: i32,
    pub beatmap_cache_timeout: u64,
    pub auto_clean_cache: bool,
    pub auto_clean_interval: u64,
    pub auto_pp_recalculate: AutoPPRecalculate,
    pub server: Server,
    pub logger: Logger,
    #[serde(rename = "prometheus")]
    pub prom: Prometheus,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
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
    pub server_log_format: String,
    pub exclude_endpoints: Vec<String>,
    pub exclude_endpoints_regex: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Prometheus {
    pub namespace: String,
    pub endpoint: String,
    pub exclude_endpoint_log: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutoPPRecalculate {
    pub interval: u64,
    pub max_retry: i32,
}
