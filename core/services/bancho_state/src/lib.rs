#[macro_use]
extern crate peace_logs;

#[allow(unused_imports)]
#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate serde;

pub mod components;
pub mod error;
pub mod services;

pub use components::*;
pub use error::*;
pub use services::*;

pub mod rpc_config {
    use clap_serde_derive::ClapSerde;
    use peace_cfg::macro_define_rpc_client_config;
    use peace_pb::bancho_state;

    macro_define_rpc_client_config!(
        service_name: bancho_state,
        config_name: BanchoStateRpcConfig,
        default_uri: "http://127.0.0.1:5011"
    );
}
pub use rpc_config::*;
