use config::Config;

use super::model::Logger;
use log::LevelFilter;

impl Logger {
    /// Init logger
    pub fn init(cfg: &Config) {
        let log_level = cfg.get_str("logger.level").unwrap_or("debug".to_string());
        let exclude_modules: Vec<String> = cfg.get("logger.exclude_modules").unwrap_or_default();

        let env = env_logger::Env::default().filter_or(
            "LOG_FILTER",
            cfg.get_str(&format!("logger.mode.{}", &log_level))
                .unwrap_or("info".to_string()),
        );

        let mut builder = env_logger::Builder::from_env(env);
        for module in &exclude_modules {
            builder.filter_module(module, LevelFilter::Warn);
        }
        builder.format_timestamp_millis().init();
    }
}
