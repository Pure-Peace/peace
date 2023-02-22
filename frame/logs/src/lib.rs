#[cfg(any(feature = "grpc", feature = "api_axum"))]
#[macro_use]
extern crate tracing;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "api_axum")]
pub mod api;

use clap::Parser;
use clap_serde_derive::ClapSerde;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
pub use tracing::log::LevelFilter;
pub use tracing::*;
pub use tracing_subscriber::*;
pub use tracing_subscriber::{
    fmt, layer::Layered, prelude::*, reload, reload::Handle, reload::Layer,
    Registry,
};

pub use crate::macro_impl_logging_config as impl_logging_config;

pub struct ReloadHandles<
    R = Registry,
    L = fmt::Layer<R>,
    S = Layered<Layer<L, R>, R>,
> {
    pub fmt_reload: Handle<L, R>,
    pub env_filter_reload: Handle<EnvFilter, S>,
}

/// Get [`EnvFilter`] (set new env if `set_env` not `None`).
///
/// ```rust
/// use peace_logs;
/// peace_logs::env_filter(Some("ENV"));
/// let _ = peace_logs::tracing();
///
pub fn env_filter(set_env: Option<&str>) -> EnvFilter {
    const ALWAYS: &'static str = ",rustls::conn=off";
    static ENV: OnceCell<Arc<Mutex<String>>> = OnceCell::new();
    let s = ENV.get_or_init(|| {
        Arc::new(Mutex::new(set_env.unwrap_or("").to_string()))
    });
    let mut sl = s.lock().unwrap();
    if let Some(set) = set_env {
        *sl = set.to_string();
    }
    EnvFilter::from_str(&format!("{}{}", sl, ALWAYS)).unwrap()
}

/// Init tracing and returns [`ReloadHandles`] for runtime reloading.
///
/// ```rust
/// use peace_logs;
///
/// peace_logs::set_level(peace_logs::log::LevelFilter::Info);
/// let reload_handles = peace_logs::tracing();
/// reload_handles.env_filter_reload.reload("debug").unwrap();
/// ```
///
pub fn tracing() -> &'static ReloadHandles {
    static TRACING: OnceCell<ReloadHandles> = OnceCell::new();
    TRACING.get_or_init(|| {
        let (fmt, fmt_reload) = Layer::new(fmt::Layer::new());
        let (env_filter, env_filter_reload) = Layer::new(env_filter(None));

        tracing_subscriber::registry().with(fmt).with(env_filter).init();

        ReloadHandles { fmt_reload, env_filter_reload }
    })
}

/// Set logging level with [`LevelFilter`].
///
/// ```rust
/// use peace_logs;
///
/// let _ = peace_logs::tracing();
/// peace_logs::set_level(peace_logs::log::LevelFilter::Warn).unwrap();
/// ```
///
pub fn set_level(level: LevelFilter) -> Result<(), reload::Error> {
    tracing()
        .env_filter_reload
        .reload(env_filter(Some(&level.to_string())))
        .unwrap();
    Ok(())
}

/// Reload [`LevelFilter`].
///
/// ```rust
/// use peace_logs;
///
/// let _ = peace_logs::tracing();
/// peace_logs::set_env_filter("ENV").unwrap();
/// ```
///
pub fn set_env_filter(filter: &str) -> Result<(), reload::Error> {
    tracing().env_filter_reload.reload(env_filter(Some(filter))).unwrap();
    Ok(())
}

/// Toggle debug (verbose) mode.
///
/// Turning on debug will display information such as code line number, source file, thread id, etc.
pub fn toggle_debug_mode(enabled: bool) -> Result<(), reload::Error> {
    let handles = tracing();
    let layer = fmt::Layer::new();
    handles.fmt_reload.reload(if enabled {
        layer
            .with_line_number(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_line_number(true)
            .with_file(true)
    } else {
        layer
    })
}

pub trait IntoLevelFilter {
    fn into_level_filter(&self) -> Result<LevelFilter, &str>;
}

impl IntoLevelFilter for i32 {
    /// Convert [`i32`] to [`LevelFilter`]
    fn into_level_filter(&self) -> Result<LevelFilter, &str> {
        match *self {
            0 => Ok(LevelFilter::Off),
            1 => Ok(LevelFilter::Error),
            2 => Ok(LevelFilter::Warn),
            3 => Ok(LevelFilter::Info),
            4 => Ok(LevelFilter::Debug),
            5 => Ok(LevelFilter::Trace),
            _ => Err("Invalid level"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[cfg_attr(feature = "openapi_axum", derive(utoipa::ToSchema))]
pub enum LogLevel {
    /// Designates that trace instrumentation should be completely disabled.
    Off,
    /// Designates very serious errors.
    Error,
    /// Designates hazardous situations.
    Warn,
    /// Designates useful information.
    Info,
    /// Designates lower priority information.
    Debug,
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

#[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfigArgs {
    /// Logging level.
    #[default(LogLevel::Info)]
    #[arg(short = 'L', long, value_enum, default_value = "info")]
    pub log_level: LogLevel,

    /// Logging env filter.
    #[arg(short = 'F', long)]
    pub log_env_filter: Option<String>,

    /// Turning on debug will display information such as code line number, source file, thread id, etc.
    #[default(false)]
    #[arg(short, long)]
    pub debug: bool,
}

pub trait LoggingConfig {
    fn log_level(&self) -> LogLevel;
    fn env_filter(&self) -> Option<String>;
    fn debug(&self) -> bool;
}

/// Configure with [`LoggingConfig`].
pub fn init(cfg: &impl LoggingConfig) {
    env_filter(Some(&format!(
        "{},{}",
        LevelFilter::from(cfg.log_level()),
        cfg.env_filter().unwrap_or("".to_string())
    )));

    // Init logging
    tracing();
    if cfg.debug() {
        toggle_debug_mode(true).unwrap();
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! macro_impl_logging_config {
        ($t: ty) => {
            impl $crate::LoggingConfig for $t {
                fn log_level(&self) -> $crate::LogLevel {
                    self.logging.log_level
                }

                fn env_filter(&self) -> Option<String> {
                    self.logging.log_env_filter.clone()
                }

                fn debug(&self) -> bool {
                    self.logging.debug
                }
            }
        };
    }
}
