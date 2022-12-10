pub use clap::Parser;
pub use clap_serde_derive::ClapSerde;
pub use paste;
pub use peace_logs::{log::LevelFilter, LogLevel};
pub use sea_orm::{ConnectOptions, DatabaseConnection, DbErr};
pub use serde::{Deserialize, Serialize};

/// Create a configuration structure for the specified database.
///
/// [clap #3513](https://github.com/clap-rs/clap/issues/3513)
/// Due to `clap` currently does not support adding a prefix to the configuration struct,
/// this declarative macro is currently used to reuse configuration items.
///
/// examples
///
/// ```rust
/// use peace_dal::define_db_config;
///
/// define_db_config!(BanchoDbConfig, bancho);
/// define_db_config!(LazerDbConfig, lazer);
/// ```
#[macro_export]
macro_rules! define_db_config {
    ($struct_name: ident, $prefix: ident) => {
        $crate::macros::paste::paste! {
            #[allow(non_snake_case)]
            mod [<__def_ $struct_name _mod__>] {
                use $crate::macros::{Parser, ClapSerde, Deserialize, Serialize};

                /// Database configurations
                #[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize)]
                pub struct $struct_name {
                    /// Database connection URL.
                    #[default("protocol://username:password@host/database".to_string())]
                    #[arg(
                        long,
                        default_value = "protocol://username:password@host/database"
                    )]
                    pub [<$prefix _db_url>]: String,

                    /// Set the maximum number of connections of the pool.
                    #[arg(long)]
                    pub [<$prefix _db_max_connections>]: Option<u32>,

                    /// Set the minimum number of connections of the pool.
                    #[arg(long)]
                    pub [<$prefix _db_min_connections>]: Option<u32>,

                    /// Set the timeout duration when acquiring a connection.
                    #[arg(long)]
                    pub [<$prefix _db_connect_timeout>]: Option<u64>,

                    /// Set the maximum amount of time to spend waiting for acquiring a connection.
                    #[arg(long)]
                    pub [<$prefix _db_acquire_timeout>]: Option<u64>,

                    /// Set the idle duration before closing a connection.
                    #[arg(long)]
                    pub [<$prefix _db_idle_timeout>]: Option<u64>,

                    /// Set the maximum lifetime of individual connections.
                    #[arg(long)]
                    pub [<$prefix _db_max_lifetime>]: Option<u64>,

                    /// Enable SQLx statement logging (default true).
                    #[default(true)]
                    #[arg(long, default_value = "true")]
                    pub [<$prefix _db_sqlx_logging>]: bool,

                    /// Set SQLx statement logging level (default [`LogLevel::Info`]) (ignored if `sqlx_logging` is false).
                    #[default($crate::macros::LogLevel::Info)]
                    #[arg(long, value_enum, default_value = "info")]
                    pub [<$prefix _db_sqlx_logging_level>]: $crate::macros::LogLevel,

                    /// Set schema search path (PostgreSQL only).
                    #[arg(long)]
                    pub [<$prefix _db_set_schema_search_path>]: Option<String>,
                }

                $crate::impl_db_config!($struct_name, $prefix);
            }

            pub use [<__def_ $struct_name _mod__>]::$struct_name;
        }
    };
}

#[macro_export]
macro_rules! impl_db_config {
    ($struct_name: ident, $prefix: ident) => {
        $crate::macros::paste::paste! {
            impl $crate::DbConfig for $struct_name {
                fn configured_opt(&self) -> $crate::macros::ConnectOptions {
                    let mut opt = $crate::macros::ConnectOptions::new(self.[<$prefix _db_url>].clone());

                    if let Some(v) = self.[<$prefix _db_max_connections>] {
                        opt.max_connections(v);
                    }
                    if let Some(v) = self.[<$prefix _db_min_connections>] {
                        opt.min_connections(v);
                    }
                    if let Some(v) =
                        self.[<$prefix _db_connect_timeout>].map(std::time::Duration::from_secs)
                    {
                        opt.connect_timeout(v);
                    }
                    if let Some(v) =
                        self.[<$prefix _db_acquire_timeout>].map(std::time::Duration::from_secs)
                    {
                        opt.acquire_timeout(v);
                    }
                    if let Some(v) =
                        self.[<$prefix _db_idle_timeout>].map(std::time::Duration::from_secs)
                    {
                        opt.idle_timeout(v);
                    }
                    if let Some(v) =
                        self.[<$prefix _db_max_lifetime>].map(std::time::Duration::from_secs)
                    {
                        opt.max_lifetime(v);
                    }
                    if let Some(v) = self.[<$prefix _db_set_schema_search_path>].to_owned() {
                        opt.set_schema_search_path(v);
                    }
                    opt.sqlx_logging(self.[<$prefix _db_sqlx_logging>]);
                    opt.sqlx_logging_level($crate::macros::LevelFilter::from(
                        self.[<$prefix _db_sqlx_logging_level>],
                    ));

                    opt
                }
            }
        }
    };
}