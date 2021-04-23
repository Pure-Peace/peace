use colored::Colorize;
use config::{Config, ConfigError, /* Environment, */ File};
use dotenv::dotenv;
use peace_constants::{PEACE_BANNER, PEACE_LOCAL_CONFIG_DIR};
use std::env;

use super::LocalConfigData;
use super::Logger;

#[derive(Debug, Clone)]
pub struct LocalConfig {
    pub env: String,
    pub cfg: Config,
    pub data: LocalConfigData,
}

impl LocalConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Print banner
        println!("{}", PEACE_BANNER.green());

        // Start loading
        println!("{}", "> Start loading local config!".green());
        let env = Self::load_env();
        let cfg = Self::load_settings(env.clone())?;
        let data: LocalConfigData = cfg.clone().try_into()?;
        println!(
            "{}",
            "> Configuration loaded successfully!\n".bold().green()
        );
        // You can deserialize (and thus freeze) the entire configuration as cfg.try_into()
        Ok(LocalConfig { env, cfg, data })
    }

    pub fn init() -> Self {
        let cfg = LocalConfig::new();
        if let Err(err) = cfg {
            error!("Settings failed to initialize, please check the local configuration file! Error: {:?}", err);
            panic!();
        }
        cfg.unwrap()
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
        println!("{}", "> Loading local config file...".green());

        // The "default" configuration file
        cfg.merge(File::with_name(&format!(
            "{}/default",
            PEACE_LOCAL_CONFIG_DIR
        )))
        .expect(&format!(
            "could not findout default config file, path: {}/default",
            PEACE_LOCAL_CONFIG_DIR
        ));

        // Add in the current environment file
        cfg.merge(File::with_name(&format!("{}/{}", PEACE_LOCAL_CONFIG_DIR, env)).required(true))
            .expect(
                &format!("Please make sure that the configuration file of the current environment exists in ./{}/{}", PEACE_LOCAL_CONFIG_DIR, env),
            );

        // Initial logger
        println!("{}", "> Initializing logger...".green());
        Logger::init(&cfg);

        // Set the server addr
        let server: &[String; 2] = &[cfg.get("server.host")?, cfg.get("server.port")?];
        cfg.set("server.addr", format!("{}:{}", server[0], server[1]))?;

        // Example: Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        // cfg.merge(Environment::with_prefix("app"))?;

        Ok(cfg)
    }
}
