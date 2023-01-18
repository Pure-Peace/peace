use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SingletonConfig)]
/// Add a `get` method to the given struct.
///
/// This method only initializes the configuration struct when it is called for the first time,
/// and subsequent calls will directly return the [`std::sync::Arc<T>`] of the struct.
///
/// ```rust,ignore
/// use clap::Parser;
/// use clap_serde_derive::ClapSerde;
/// use serde::{Deserialize, Serialize};
/// use peace_cfg::{SingletonConfig, impl_config};
///
/// #[derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize, SingletonConfig)]
/// pub struct GatewayConfig {
///     /// A list of hostnames to route to the bancho service.
///     #[arg(short = 'B', long)]
///     pub bancho_hostname: Vec<String>,
/// }
///
///
/// fn main() {
///     GatewayConfig::get();
/// }
/// ```
pub fn derive_singleton_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    match input.data {
        syn::Data::Struct(data) => data.fields,
        _ => panic!("#[derive(SingletonConfig)] is only defined for structs."),
    };

    let expanded = quote! {
        impl_config!(#name);
    };

    expanded.into()
}

#[proc_macro_attribute]
/// Including derives: `derive(Parser, ClapSerde, Debug, Clone, Serialize, Deserialize, SingletonConfig)`
///
/// Example:
/// ```rust,ignore
/// use peace_cfg::{SingletonConfig, impl_config};
/// use clap_serde_derive::ClapSerde;
///
/// /// Command Line Interface (CLI) for DB service.
/// #[peace_config]
/// #[command(name = "db", author, version, about, propagate_version = true)]
/// pub struct DbServiceConfig {
///     #[command(flatten)]
///     pub frame_cfg: RpcFrameConfig,
///
///     #[command(flatten)]
///     pub peace_db: PeaceDbConfig,
/// }
/// ```
pub fn peace_config(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    return quote! {
        #[derive(
            clap::Parser,
            clap_serde_derive::ClapSerde,
            core::fmt::Debug,
            core::clone::Clone,
            serde::Serialize,
            serde::Deserialize,
            SingletonConfig,
        )]
        #input
    }
    .into();
}
