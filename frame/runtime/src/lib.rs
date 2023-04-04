pub mod cfg;

use std::time::Duration;

use cfg::RuntimeConfig;
use tokio::runtime::{self, Builder, Runtime};

pub trait CreateRuntime {
    fn builder(&self) -> Builder;
    fn runtime(&self) -> Result<Runtime, std::io::Error>;
}

pub fn builder(cfg: &RuntimeConfig) -> Builder {
    let mut builder = if cfg.runtime_multi_thread {
        runtime::Builder::new_multi_thread()
    } else {
        runtime::Builder::new_current_thread()
    };

    if cfg.runtime_enable_io {
        builder.enable_io();
    }
    if cfg.runtime_enable_time {
        builder.enable_time();
    }
    if cfg.runtime_enable_all {
        builder.enable_all();
    }
    if let Some(val) = cfg.runtime_event_interval {
        builder.event_interval(val);
    }
    if let Some(val) = cfg.runtime_global_queue_interval {
        builder.global_queue_interval(val);
    }
    if let Some(val) = cfg.runtime_max_blocking_threads {
        builder.max_blocking_threads(val);
    }
    if let Some(val) = cfg.runtime_thread_keep_alive {
        builder.thread_keep_alive(Duration::from_millis(val));
    }
    if let Some(val) = cfg.runtime_thread_name.as_ref() {
        builder.thread_name(val);
    }
    if let Some(val) = cfg.runtime_thread_stack_size {
        builder.thread_stack_size(val);
    }
    if let Some(val) = cfg.runtime_worker_threads {
        builder.worker_threads(val);
    }

    builder
}

pub fn runtime(
    cfg: &RuntimeConfig,
) -> Result<tokio::runtime::Runtime, std::io::Error> {
    builder(cfg).build()
}
