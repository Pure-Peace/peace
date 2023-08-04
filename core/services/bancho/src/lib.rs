#[macro_use]
extern crate peace_logs;

#[allow(unused_imports)]
#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate serde;

pub mod error;
pub mod services;

pub use error::*;
pub use services::*;

pub mod rpc_config {
    use clap_serde_derive::ClapSerde;
    use pb_bancho as bancho;
    use peace_cfg::macro_define_rpc_client_config;

    macro_define_rpc_client_config!(
        service_name: bancho,
        config_name: BanchoRpcConfig,
        default_uri: "http://127.0.0.1:5010"
    );
}
pub use rpc_config::*;
