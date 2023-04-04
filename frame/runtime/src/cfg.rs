use crate::CreateRuntime;
use clap_serde_derive::ClapSerde;
use peace_cfg::{impl_config, peace_config, SingletonConfig};
use tokio::runtime::Builder;

#[peace_config]
pub struct RuntimeConfig {
    /// Enable multi thread (if feature is included)
    #[default(false)]
    #[arg(short = 'M', long)]
    pub runtime_multi_thread: bool,

    /// Sets the number of worker threads the Runtime will use.
    ///
    /// This can be any number above 0 though it is advised to keep this value on the smaller side.
    ///
    /// The default value is the number of cores available to the system.
    ///
    /// When using the current_thread runtime this method has no effect.
    #[arg(long)]
    pub runtime_worker_threads: Option<usize>,

    /// Sets name of threads spawned by the Runtime's thread pool.
    ///
    /// The default name is "tokio-runtime-worker".
    #[arg(long)]
    pub runtime_thread_name: Option<String>,

    /// Sets the stack size (in bytes) for worker threads.
    ///
    /// The actual stack size may be greater than this value if
    /// the platform specifies minimal stack size.
    ///
    /// The default stack size for spawned threads is 2 MiB,
    /// though this particular stack size is subject to change in the future.
    #[arg(long)]
    pub runtime_thread_stack_size: Option<usize>,

    /// Enables the I/O driver.
    ///
    /// Doing this enables using net, process, signal, and some I/O types on the runtime.
    #[default(true)]
    #[arg(long, default_value = "true")]
    pub runtime_enable_io: bool,

    /// Enables the time driver.
    ///
    /// Doing this enables using `tokio::time` on the runtime.
    #[default(true)]
    #[arg(long, default_value = "true")]
    pub runtime_enable_time: bool,

    /// Enables both I/O and time drivers.
    ///
    /// Doing this is a shorthand for calling enable_io and enable_time individually.
    /// If additional components are added to Tokio in the future,
    /// enable_all will include these future components.
    #[default(true)]
    #[arg(long, default_value = "true")]
    pub runtime_enable_all: bool,

    /// Sets the number of scheduler ticks after which the scheduler will poll for external events (timers, I/O, and so on).

    /// A scheduler "tick" roughly corresponds to one `poll` invocation on a task.
    ///
    /// By default, the event interval is `61` for all scheduler types.
    ///
    /// Setting the event interval determines the effective "priority" of
    /// delivering these external events (which may wake up additional tasks),
    /// compared to executing tasks that are currently ready to run.
    /// A smaller value is useful when tasks frequently spend a long time in polling,
    /// or frequently yield, which can result in overly long delays picking up I/O events.
    /// Conversely, picking up new events requires extra synchronization and syscall overhead,
    /// so if tasks generally complete their polling quickly,
    /// a higher event interval will minimize that overhead while still keeping the scheduler responsive to events.
    #[arg(long)]
    pub runtime_event_interval: Option<u32>,

    /// Sets the number of scheduler ticks after which the scheduler will poll the global task queue.
    ///
    /// A scheduler "tick" roughly corresponds to one `poll` invocation on a task.
    ///
    /// By default the global queue interval is:
    ///
    /// - `31` for the current-thread scheduler.
    /// - `61` for the multithreaded scheduler.
    ///
    /// Schedulers have a local queue of already-claimed tasks, and a global queue of incoming tasks.
    /// Setting the interval to a smaller value increases the fairness of the scheduler,
    /// at the cost of more synchronization overhead.
    /// That can be beneficial for prioritizing getting started on new work,
    /// especially if tasks frequently yield rather than complete or await on further I/O.
    /// Conversely, a higher value prioritizes existing work,
    /// and is a good choice when most tasks quickly complete polling.
    #[arg(long)]
    pub runtime_global_queue_interval: Option<u32>,

    /// Specifies the limit for additional threads spawned by the Runtime.
    ///
    /// These threads are used for blocking operations like tasks spawned through `[spawn_blocking]`.
    /// Unlike the `[worker_threads]`, they are not always active and will exit if left idle for too long.
    /// You can change this timeout duration with `[thread_keep_alive]`.
    ///
    /// The default value is 512.
    #[arg(long)]
    pub runtime_max_blocking_threads: Option<usize>,

    /// Sets a custom timeout for a thread in the blocking pool (millis).
    ///
    /// By default, the timeout for a thread is set to 10 seconds. This can be overridden using .thread_keep_alive().
    #[arg(long)]
    pub runtime_thread_keep_alive: Option<u64>,
}

impl CreateRuntime for RuntimeConfig {
    fn builder(&self) -> Builder {
        crate::builder(&self)
    }
    fn runtime(&self) -> Result<tokio::runtime::Runtime, std::io::Error> {
        crate::runtime(&self)
    }
}
