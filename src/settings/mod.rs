pub mod actix;
pub mod types;

use dotenv::dotenv;
use config::{Config, ConfigError, /* Environment, */ File};
use std::env;
use types::*;

impl Settings {
    pub fn new() -> Result<(Config, Self), ConfigError> {
        // Load .env
        dotenv().ok();
        
        let mut cfg = Config::new();

        // Current env
        // Default to 'development' env
        // Args > .env
        let env = match env::args().nth(1) {
            None => env::var("RUN_MODE").unwrap_or_else(|_| "development".into()),
            Some(any) => any,
        };

        // Set the env
        cfg.set("env", env.clone())?;

        // The "default" configuration file
        cfg.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        cfg.merge(File::with_name(&format!("config/{}", env)).required(true))
            .expect(
                "Please make sure that the configuration file of the current environment exists",
            );

        // Set the addr
        let (host, port): (String, String) = (cfg.get("server.host")?, cfg.get("server.port")?);
        cfg.set("server.addr", format!("{}:{}", host, port))?;
        // Example: Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        // cfg.merge(Environment::with_prefix("app"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        Ok((cfg.clone(), cfg.try_into()?))
    }
}
