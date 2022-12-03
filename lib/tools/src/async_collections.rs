use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::{signal, sync::Notify};

#[derive(Debug, Default)]
pub struct NotifyOnce {
    notified: AtomicBool,
    notify: Notify,
}

impl NotifyOnce {
    pub fn notify_waiters(&self) {
        self.notified.store(true, Ordering::SeqCst);

        self.notify.notify_waiters();
    }

    pub async fn notified(&self) {
        let future = self.notify.notified();

        if !self.notified.load(Ordering::SeqCst) {
            future.await;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SignalHandle(Arc<NotifyOnce>);

impl SignalHandle {
    /// Create a new handle.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trigger(&self) {
        self.0.notify_waiters();
    }

    pub async fn wait_signal(&self) {
        self.0.notified().await;
    }
}

pub async fn shutdown_signal<F>(shutdown: F)
where
    F: Fn(&str) -> (),
{
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    let s = tokio::select! {
        _ = ctrl_c => "CONTROL_C",
        _ = terminate => "TERMINATE",
    };

    shutdown(s);
}
