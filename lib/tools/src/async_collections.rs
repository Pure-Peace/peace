use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::{signal, sync::Notify, task::JoinHandle};

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

#[derive(thiserror::Error, Debug)]
pub enum BackgroundServiceError {
    #[error("task is already started")]
    AlreadyStarted,
    #[error("task is not started")]
    NotStarted,
    #[error("signal is not exists")]
    SignalNotExists,
    #[error("failed to trigger signal")]
    FailedToTriggerSignal,
}

#[derive(Debug)]
pub struct BackgroundService<T, F>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
    F: Fn(SignalHandle) -> T,
{
    factory: F,
    handle: Option<JoinHandle<Option<T::Output>>>,
    signal: Option<SignalHandle>,
}

impl<T, F> BackgroundService<T, F>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
    F: Fn(SignalHandle) -> T,
{
    pub fn new(factory: F) -> Self {
        Self { factory, handle: None, signal: None }
    }

    pub fn is_started(&self) -> bool {
        match self.handle.as_ref() {
            Some(h) => !h.is_finished(),
            None => false,
        }
    }

    pub fn start(
        &mut self,
        manual_stop: bool,
    ) -> Result<(), BackgroundServiceError> {
        if self.is_started() {
            return Err(BackgroundServiceError::AlreadyStarted);
        }

        let signal = SignalHandle::new();
        self.signal = Some(signal.clone());

        let fut = (self.factory)(signal.clone());

        self.handle = Some(tokio::spawn(async move {
            if manual_stop {
                Some(fut.await)
            } else {
                tokio::select! {
                    v = fut => Some(v),
                    _ = signal.wait_signal() => None
                }
            }
        }));
        Ok(())
    }

    pub fn trigger_signal(&self) -> Result<(), BackgroundServiceError> {
        self.signal
            .as_ref()
            .and_then(|s| Some(s.trigger()))
            .ok_or(BackgroundServiceError::SignalNotExists)?;
        Ok(())
    }

    pub fn abort(&mut self) {
        if let Some(h) = self.handle.as_ref() {
            h.abort()
        }
    }

    pub fn handle(&mut self) -> Option<JoinHandle<Option<T::Output>>> {
        std::mem::replace(&mut self.handle, None)
    }
}
