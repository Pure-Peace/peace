use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::{signal, sync::Notify, task::JoinHandle};

/// A struct that can be used to notify a group of waiters exactly once.
#[derive(Debug, Default)]
pub struct NotifyOnce {
    notified: AtomicBool,
    notify: Notify,
}

impl NotifyOnce {
    /// Notifies all waiters that are currently waiting for the notification.
    pub fn notify_waiters(&self) {
        self.notified.store(true, Ordering::SeqCst);

        self.notify.notify_waiters();
    }

    /// Waits until the notification is received.
    pub async fn notified(&self) {
        let future = self.notify.notified();

        // If the notification has already been received, return immediately.
        if !self.notified.load(Ordering::SeqCst) {
            future.await;
        }
    }
}

/// A handle to a signal that can be triggered to notify waiters.
#[derive(Clone, Debug, Default)]
pub struct SignalHandle(Arc<NotifyOnce>);

impl SignalHandle {
    /// Creates a new `SignalHandle`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Triggers the signal to notify waiters.
    pub fn trigger(&self) {
        self.0.notify_waiters();
    }

    /// Waits until the signal is triggered.
    pub async fn wait_signal(&self) {
        self.0.notified().await;
    }
}

/// This function installs signal handlers for Ctrl+C and terminate signals,
/// and invokes the given `shutdown` function with a string indicating which
/// signal was received.
///
/// # Arguments
///
/// * `shutdown`: A function that will be invoked with a string argument
///   indicating which signal was received. This string will be either
///   "CONTROL_C" or "TERMINATE".
///
/// # Example
///
/// ```
/// async fn my_shutdown_function(signal: &str) {
///     println!("Received signal: {}", signal);
/// }
///
/// shutdown_signal(my_shutdown_function).await;
/// ```
///
/// # Panics
///
/// This function may panic if the Ctrl+C or terminate signal handlers cannot
/// be installed.
///
/// # Note
///
/// This function is only available on Unix platforms.
pub async fn shutdown_signal<F>(shutdown: F)
where
    F: Fn(&str) -> (),
{
    // Install the Ctrl+C signal handler
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    // Install the terminate signal handler
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // If we're not on Unix, create a future that never resolves
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either the Ctrl+C or terminate signal to be received
    let s = tokio::select! {
        _ = ctrl_c => "CONTROL_C",
        _ = terminate => "TERMINATE",
    };

    // Invoke the shutdown function with the signal string
    shutdown(s);
}

#[derive(thiserror::Error, Debug)]
/// An error enum to represent errors that can occur while using BackgroundService.
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
    /// A factory function that creates the future to be executed by the
    /// background service.
    factory: F,

    /// The join handle of the background service task.
    handle: Option<JoinHandle<Option<T::Output>>>,

    /// The signal handle used to gracefully stop the background service.
    signal: Option<SignalHandle>,
}

impl<T, F> BackgroundService<T, F>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
    F: Fn(SignalHandle) -> T,
{
    /// Creates a new `BackgroundService` instance with the specified factory
    /// function.
    pub fn new(factory: F) -> Self {
        Self { factory, handle: None, signal: None }
    }

    /// Returns true if the background service is started, false otherwise.
    pub fn is_started(&self) -> bool {
        match self.handle.as_ref() {
            Some(h) => !h.is_finished(),
            None => false,
        }
    }

    /// Starts the background service.
    ///
    /// If `manual_stop` is true, the future returned by the factory function
    /// will be executed until completion, even if a signal to stop the service
    /// is received.
    ///
    /// If `manual_stop` is false, the future returned by the factory function
    /// will be executed until a signal to stop the service is received.
    pub fn start(
        &mut self,
        manual_stop: bool,
    ) -> Result<(), BackgroundServiceError> {
        // Check if the background service is already started
        if self.is_started() {
            return Err(BackgroundServiceError::AlreadyStarted);
        }

        // Create a signal handle and store it in the `signal` field
        let signal = SignalHandle::new();
        self.signal = Some(signal.clone());

        // Call the factory function with the signal handle as an argument to
        // create the future to be executed by the background service
        let fut = (self.factory)(signal.clone());

        // Spawn a new task to execute the future
        self.handle = Some(tokio::spawn(async move {
            if manual_stop {
                // If manual stop is enabled, await the future until completion
                Some(fut.await)
            } else {
                // If manual stop is disabled, wait for either the future to
                // complete or a signal to stop the service to be received
                tokio::select! {
                    v = fut => Some(v),
                    _ = signal.wait_signal() => None
                }
            }
        }));
        Ok(())
    }

    /// Triggers a signal to stop the background service.
    pub fn trigger_signal(&self) -> Result<(), BackgroundServiceError> {
        // Trigger the signal handle, if it exists
        self.signal
            .as_ref()
            .and_then(|s| Some(s.trigger()))
            .ok_or(BackgroundServiceError::SignalNotExists)?;
        Ok(())
    }

    /// Aborts the background service task if it is running.
    pub fn abort(&mut self) {
        if let Some(h) = self.handle.as_ref() {
            h.abort()
        }
    }

    /// Returns the join handle of the background service task and replaces
    /// it with `None`.
    pub fn handle(&mut self) -> Option<JoinHandle<Option<T::Output>>> {
        std::mem::replace(&mut self.handle, None) // Replace the handle with None and return it.
    }
}
