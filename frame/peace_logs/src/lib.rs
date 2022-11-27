#[cfg(any(feature = "grpc", feature = "api_axum"))]
#[macro_use]
extern crate tracing;

#[cfg(feature = "grpc")]
pub mod logs_rpc {
    tonic::include_proto!("logs_rpc");
}

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "api_axum")]
pub mod api;

use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
pub use tracing::*;
pub use tracing_subscriber::*;

use once_cell::sync::OnceCell;
use tracing_subscriber::{
    fmt, layer::Layered, prelude::*, reload, reload::Handle, reload::Layer,
    Registry,
};

use tracing::level_filters::LevelFilter;

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
/// peace_logs::default_level(Some(peace_logs::level_filters::LevelFilter::INFO));
/// let handles = peace_logs::tracing();
/// handles.fmt_reload.reload(peace_logs::fmt::layer().with_file(true).with_line_number(true)).unwrap();
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
/// let _ = peace_logs::tracing();
/// peace_logs::set_level(peace_logs::level_filters::LevelFilter::WARN).unwrap();
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

/// Convert [`i32`] to [`LevelFilter`]
pub fn level_from_int(level: i32) -> Result<LevelFilter, ()> {
    match level {
        0 => Ok(LevelFilter::OFF),
        1 => Ok(LevelFilter::ERROR),
        2 => Ok(LevelFilter::WARN),
        3 => Ok(LevelFilter::INFO),
        4 => Ok(LevelFilter::DEBUG),
        5 => Ok(LevelFilter::TRACE),
        _ => Err(()),
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
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

pub trait LoggerConfig {
    fn log_level(&self) -> LogLevel;
    fn env_filter(&self) -> Option<String>;
    fn debug(&self) -> bool;
}

/// Configure with [`LoggerConfig`].
pub fn init(cfg: &impl LoggerConfig) {
    env_filter(Some(&format!(
        "{},{}",
        LevelFilter::from(cfg.log_level()),
        cfg.env_filter().unwrap_or("".to_string())
    )));

    // Init logger
    tracing();
    if cfg.debug() {
        toggle_debug_mode(true).unwrap();
    }
}
