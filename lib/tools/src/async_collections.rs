use arc_swap::ArcSwapOption;
use std::{
    future::Future,
    pin::Pin,
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
/// An error enum to represent errors that can occur while using BackgroundTask.
pub enum BackgroundTaskError {
    #[error("task is already started")]
    AlreadyStarted,
    #[error("task is not started")]
    NotStarted,
    #[error("signal is not exists")]
    SignalNotExists,
    #[error("failed to trigger signal")]
    FailedToTriggerSignal,
}

pub struct BackgroundTaskFactory {
    boxed_function: Arc<
        dyn Fn(
            SignalHandle,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    >,
}

impl BackgroundTaskFactory {
    pub fn new(
        boxed_function: Arc<
            dyn Fn(
                SignalHandle,
            )
                -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
        >,
    ) -> Self {
        Self { boxed_function }
    }

    pub fn new_future(
        &self,
        signal: SignalHandle,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        (self.boxed_function)(signal)
    }
}

#[derive(Clone, Default)]
pub struct BackgroundTaskManager {
    task: Arc<ArcSwapOption<BackgroundTask>>,
}

impl BackgroundTaskManager {
    pub fn new() -> Self {
        Self { task: Arc::new(ArcSwapOption::empty()) }
    }

    pub fn start(&self, factory: BackgroundTaskFactory, manual_stop: bool) {
        if self.task.load().is_some() {
            return;
        }

        self.task
            .store(Some(Arc::new(BackgroundTask::start(factory, manual_stop))));
    }

    pub fn stop(
        &self,
    ) -> Result<Option<Arc<BackgroundTask>>, BackgroundTaskError> {
        if let Some(task) = self.task.load_full() {
            task.trigger_signal()?;
            return Ok(self.task.swap(None));
        }

        Ok(None)
    }

    pub fn task(&self) -> &Arc<ArcSwapOption<BackgroundTask>> {
        &self.task
    }
}

#[derive(Clone)]
pub struct BackgroundTask {
    /// The join handle of the background service task.
    handle: Option<Arc<JoinHandle<Option<()>>>>,

    /// The signal handle used to gracefully stop the background service.
    signal: Option<SignalHandle>,
}

impl BackgroundTask {
    pub fn start(factory: BackgroundTaskFactory, manual_stop: bool) -> Self {
        // Create a signal handle and store it in the `signal` field
        let signal = SignalHandle::new();
        let signal_cloned = signal.clone();

        // Call the factory function with the signal handle as an argument to
        // create the future to be executed by the background service
        let fut = factory.new_future(signal.clone());

        // Spawn a new task to execute the future
        let handle = Some(Arc::new(tokio::spawn(async move {
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
        })));

        Self { handle, signal: Some(signal_cloned) }
    }

    /// Returns true if the background service is started, false otherwise.
    pub fn is_started(&self) -> bool {
        match self.handle.as_ref() {
            Some(h) => !h.is_finished(),
            None => false,
        }
    }

    pub fn signal(&self) -> Option<SignalHandle> {
        self.signal.clone()
    }

    /// Triggers a signal to stop the background service.
    pub fn trigger_signal(&self) -> Result<(), BackgroundTaskError> {
        // Trigger the signal handle, if it exists
        self.signal
            .as_ref()
            .and_then(|s| Some(s.trigger()))
            .ok_or(BackgroundTaskError::SignalNotExists)?;
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
    pub fn handle(&mut self) -> Option<Arc<JoinHandle<Option<()>>>> {
        self.handle.clone()
    }
}
