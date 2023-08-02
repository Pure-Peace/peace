pub mod ____private {
    pub use clap::Parser;
    pub use clap_serde_derive::ClapSerde;
    pub use paste;
    pub use serde::{Deserialize, Serialize};
}

#[macro_export]
macro_rules! impl_snapshot_config {
    (config: $cfg: ty, prefix: $pf: tt) => {
        $crate::macros::____private::paste::paste! {
            impl $crate::SnapshotConfig for $cfg {
                fn snapshot_path(&self) -> &str {
                    &self.[<$pf _snapshot_path>]
                }

                fn snapshot_type(&self) -> $crate::SnapshotType {
                    self.[<$pf _snapshot_type>]
                }

                fn should_save_snapshot(&self) -> bool {
                    self.[<$pf _snapshot>]
                }

                fn should_load_snapshot(&self) -> bool {
                    self.[<$pf _load_snapshot>]
                }

                fn snapshot_expired_secs(&self) -> u64 {
                    self.[<$pf _snapshot_expired_secs>]
                }
            }
        }
    };
}

#[macro_export]
macro_rules! snapshot_config_struct {
    (config: $cfg: ty, prefix: $pf: tt) => {
        $crate::macros::____private::paste::paste! {
            #[derive(
                Debug, Clone,
                $crate::macros::____private::Parser,
                $crate::macros::____private::ClapSerde,
                $crate::macros::____private::Serialize,
                $crate::macros::____private::Deserialize
            )]
            pub struct $cfg {
                #[default(concat!("./.snapshots/", stringify!($pf) ,".snapshot").to_owned())]
                #[arg(long, default_value = concat!("./.snapshots/", stringify!($pf) ,".snapshot"))]
                pub [<$pf _snapshot_path>]: String,

                #[default(SnapshotType::Binary)]
                #[arg(long, value_enum, default_value = "binary")]
                pub [<$pf _snapshot_type>]: SnapshotType,

                #[arg(long)]
                pub [<$pf _snapshot>]: bool,

                #[arg(long)]
                pub [<$pf _load_snapshot>]: bool,

                #[default(300)]
                #[arg(long, default_value = "300")]
                pub [<$pf _snapshot_expired_secs>]: u64,
            }
        }
    };
}

#[macro_export]
macro_rules! cli_snapshot_config {
    (service: $s: tt) => {
        $crate::macros::____private::paste::paste! {
            $crate::snapshot_config_struct!(config: [<Cli $s:camel ServiceSnapshotConfigs>], prefix: [<$s:snake>]);
            $crate::impl_snapshot_config!(config: [<Cli $s:camel ServiceSnapshotConfigs>], prefix: [<$s:snake>]);
        }
    };
}
