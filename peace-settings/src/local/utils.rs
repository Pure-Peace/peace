use super::Logger;
use colored::Colorize;
use config::{Config, ConfigError, File /* Environment, */};
use dotenv::dotenv;
use std::env;

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

pub fn load_settings(env: String, config_dir: &str) -> Result<Config, ConfigError> {
    let mut cfg = Config::new();

    // Set the env
    cfg.set("env", env.clone())?;
    println!("{}", "> Loading local config file...".green());

    // The "default" configuration file
    cfg.merge(File::with_name(&format!("{}/default", config_dir)))
        .expect(&format!(
            "could not findout default config file, path: {}/default",
            config_dir
        ));

    // Add in the current environment file
    cfg.merge(File::with_name(&format!("{}/{}", config_dir, env)).required(true))
        .expect(
            &format!("Please make sure that the configuration file of the current environment exists in ./{}/{}", config_dir, env),
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
