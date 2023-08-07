#[macro_use]
extern crate peace_logs;

#[allow(unused_imports)]
#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate serde;

pub mod error;
pub mod events;
pub mod traits;

pub use error::*;
pub use events::*;
pub use traits::*;

pub mod rpc_config {
    use clap_serde_derive::ClapSerde;
    use pb_events as events;
    use peace_cfg::macro_define_rpc_client_config;

    macro_define_rpc_client_config!(
        service_name: events,
        config_name: EventsRpcConfig,
        default_uri: "http://127.0.0.1:5015"
    );
}
pub use rpc_config::*;
