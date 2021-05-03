use colored::Colorize;
use config::{Config, ConfigError /* Environment, */};

use peace_constants::common::{PEACE_BANNER, PEACE_LOCAL_CONFIG_DIR, PEACE_VERSION};

use super::LocalConfigData;

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
        println!("<Peace> version: {}. Taking off\n", PEACE_VERSION.green());
        println!("{}", "> Start loading local config!".green());
        let env = super::utils::load_env();
        let cfg = super::utils::load_settings(env.clone(), PEACE_LOCAL_CONFIG_DIR)?;
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
}
