#[cfg(any(feature = "grpc", feature = "api"))]
#[macro_use]
extern crate tracing;

#[cfg(feature = "grpc")]
pub mod logs_rpc {
    tonic::include_proto!("logs_rpc");
}

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "api")]
pub mod api;

pub use tracing::*;
pub use tracing_subscriber::*;

use once_cell::sync::OnceCell;
use tracing_subscriber::{
    fmt::Layer as LayerF, layer::Layered, prelude::*, reload, reload::Handle,
    reload::Layer as LayerR, Registry,
};

use tracing::level_filters::LevelFilter;

pub struct ReloadHandles<
    F = LevelFilter,
    R = Registry,
    L = Layered<LayerR<F, R>, R>,
> {
    pub filter_reload: Handle<F, R>,
    pub fmt_reload: Handle<LayerF<L>, L>,
}

/// Init or get default [`LevelFilter`].
///
/// ```rust
/// use peace_logs;
/// peace_logs::default_level(Some(peace_logs::level_filters::LevelFilter::INFO));
/// let _ = peace_logs::tracing();
///
pub fn default_level(level: Option<LevelFilter>) -> LevelFilter {
    static LEVEL: OnceCell<LevelFilter> = OnceCell::new();
    *LEVEL.get_or_init(|| level.unwrap_or(LevelFilter::INFO))
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
        let (filter, filter_reload) = LayerR::new(default_level(None));
        let (fmt, fmt_reload) = LayerR::new(LayerF::new());

        tracing_subscriber::registry().with(filter).with(fmt).init();

        ReloadHandles { filter_reload, fmt_reload }
    })
}

/// Reload [`LevelFilter`].
///
/// ```rust
/// use peace_logs;
/// let _ = peace_logs::tracing();
/// peace_logs::reload_level(peace_logs::level_filters::LevelFilter::WARN).unwrap();
/// ```
///
pub fn reload_level(level: LevelFilter) -> Result<(), reload::Error> {
    let handles = tracing();
    handles.filter_reload.modify(|f| *f = level)
}

/// Toggle debug (verbose) mode.
pub fn debug_mode(enabled: bool) -> Result<(), reload::Error> {
    let handles = tracing();
    let layer = LayerF::new();
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
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

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

pub trait LoggerArgs {
    fn log_level(&self) -> LogLevel;
    fn debug(&self) -> bool;
}

/// Configure with [`LoggerArgs`].
pub fn init_with_args(args: &impl LoggerArgs) {
    // Overwrite default log level
    if args.log_level() != LogLevel::Info {
        default_level(Some(args.log_level().into()));
    }
    // Init logger
    tracing();
    if args.debug() {
        debug_mode(true).unwrap();
    }
}
