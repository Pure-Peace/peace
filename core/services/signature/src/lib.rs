#[macro_use]
extern crate peace_logs;

#[allow(unused_imports)]
#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate serde;

pub mod error;
pub mod signature;
pub mod traits;

pub use signature::*;
pub use traits::*;

pub mod rpc_config {
    use clap_serde_derive::ClapSerde;
    use pb_signature as signature;
    use peace_cfg::macro_define_rpc_client_config;

    macro_define_rpc_client_config!(
        service_name: signature,
        config_name: SignatureRpcConfig,
        default_uri: "http://127.0.0.1:5014"
    );
}
pub use rpc_config::*;
