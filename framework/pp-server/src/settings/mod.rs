pub mod model;

pub const BANNER: &str = r#"

 _____  _____       _____  _____  _____  __ __  _____  _____ 
 /  _  \/  _  \ ___ /  ___>/   __\/  _  \/  |  \/   __\/  _  \
 |   __/|   __/<___>|___  ||   __||  _  <\  |  /|   __||  _  <
 \__/   \__/        <_____/\_____/\__|\_/ \___/ \_____/\__|\_/
                                                              
 
"#;
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

use colored::Colorize;

#[derive(Debug, Clone)]
pub struct LocalConfig {
    pub env: String,
    pub cfg: config::Config,
    pub data: model::LocalConfigData,
}

impl LocalConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        // Print banner
        println!("{}", BANNER.green());

        // Start loading
        println!(
            "<pp-server> version: {}. Taking off!\n",
            VERSION.unwrap_or("unknown").green()
        );
        println!("{}", "> Start loading local config!".green());
        let env = peace_settings::local::utils::load_env();
        let cfg = peace_settings::local::utils::load_settings(
            env.clone(),
            peace_constants::common::PP_SERVER_LOCAL_CONFIG_DIR,
        )?;
        let data: model::LocalConfigData = cfg.clone().try_into()?;
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
