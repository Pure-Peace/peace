use serde::Deserialize;

use colored::Colorize;
use config::{Config, ConfigError, /* Environment, */ File};
use std::env;

use dotenv::dotenv;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub postgres: deadpool_postgres::Config,
    pub redis: deadpool_redis::Config,

    pub check_pools_on_created: bool,
    pub check_db_version_on_created: bool,
    pub env: String,
    pub debug: bool,
    pub server: Server,
    pub pp_server: PPServer,
    pub geoip: Geoip,
    pub logger: Logger,
}

impl Settings {
    pub fn new() -> Result<(Config, Settings), ConfigError> {
        println!("{}", "> Start loading settings!".green());
        let env = Settings::load_env();
        let cfg = Settings::load_settings(env)?;
        println!(
            "{}",
            "> Configuration loaded successfully!\n".bold().green()
        );
        // You can deserialize (and thus freeze) the entire configuration as cfg.try_into()
        Ok((cfg.clone(), cfg.try_into()?))
    }

    pub fn load_env() -> String {
        // Load .env
        dotenv().ok();
        // Current env
        // Default to 'development' env
        // Args > .env file
        let env = match env::args().nth(1) {
            None => env::var("RUN_MODE").unwrap_or_else(|_| "development".into()),
            Some(any) => any,
        };
        println!(
            "{}",
            format!("> Current environment: {}", env.bold().yellow()).green()
        );
        env
    }

    pub fn load_settings(env: String) -> Result<Config, ConfigError> {
        let mut cfg = Config::new();

        // Set the env
        cfg.set("env", env.clone())?;
        println!("{}", "> Loading user settings...".green());

        // The "default" configuration file
        cfg.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        cfg.merge(File::with_name(&format!("config/{}", env)).required(true))
            .expect(
                "Please make sure that the configuration file of the current environment exists",
            );

        // Initial logger
        println!("{}", "> Initializing logger...".green());
        Logger::init(&cfg);

        // Set the server addr
        let server: &[String; 2] = &[cfg.get("server.host")?, cfg.get("server.port")?];
        cfg.set("server.addr", format!("{}:{}", server[0], server[1]))?;

        // Set the pp server addr
        let pp_server: &[String; 2] = &[cfg.get("pp_server.host")?, cfg.get("pp_server.port")?];
        cfg.set(
            "pp_server.addr",
            format!("{}:{}", pp_server[0], pp_server[1]),
        )?;

        // Example: Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        // cfg.merge(Environment::with_prefix("app"))?;

        Ok(cfg)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PPServer {
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
    pub actix_log_format: String,
    pub exclude_endpoints: Vec<String>,
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
