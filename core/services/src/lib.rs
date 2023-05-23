#[macro_use]
extern crate peace_logs;

#[allow(unused_imports)]
#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

pub mod bancho;
pub mod bancho_state;
pub mod chat;
pub mod gateway;
pub mod geoip;
pub mod signature;

pub mod rpc_config {
    use clap_serde_derive::ClapSerde;
    use peace_api::define_rpc_client_config;
    use peace_pb::*;

    define_rpc_client_config!(
        service_name: bancho,
        config_name: BanchoRpcConfig,
        default_uri: "http://127.0.0.1:5010"
    );

    define_rpc_client_config!(
        service_name: bancho_state,
        config_name: BanchoStateRpcConfig,
        default_uri: "http://127.0.0.1:5011"
    );

    define_rpc_client_config!(
        service_name: chat,
        config_name: ChatRpcConfig,
        default_uri: "http://127.0.0.1:5012"
    );

    define_rpc_client_config!(
        service_name: geoip,
        config_name: GeoipRpcConfig,
        default_uri: "http://127.0.0.1:5013"
    );

    define_rpc_client_config!(
        service_name: signature,
        config_name: SignatureRpcConfig,
        default_uri: "http://127.0.0.1:5014"
    );
}

pub trait FromRpcClient: RpcClient {
    fn from_client(client: Self::Client) -> Self;
}

pub trait RpcClient {
    type Client;

    fn client(&self) -> Self::Client;
}

pub trait IntoService<T>: Sized + Sync + Send + 'static {
    fn into_service(self) -> T;
}
