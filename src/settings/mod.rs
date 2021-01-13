pub mod peace;
pub mod logger;
pub mod model;

use colored::Colorize;
use config::{Config, ConfigError, /* Environment, */ File};
use dotenv::dotenv;
use model::{Logger, Settings};
use std::env;

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
        // Set the addr
        let (host, port): (String, String) = (cfg.get("server.host")?, cfg.get("server.port")?);
        cfg.set("server.addr", format!("{}:{}", host, port))?;
        // Example: Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        // cfg.merge(Environment::with_prefix("app"))?;
        Ok(cfg)
    }
}
