use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(RpcError)]
pub fn rpc_error_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_rpc_error(&ast)
}

fn impl_rpc_error(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl<'t> RpcError<'t> for #name {}

        impl std::convert::From<tonic::Status> for #name {
            fn from(status: tonic::Status) -> #name {
                match status.code() {
                    tonic::Code::Internal => status
                        .metadata()
                        .get(peace_rpc_error::SERVICE_ERROR_HEADER)
                        .and_then(|val| val.to_str().ok())
                        .and_then(|s| serde_json::from_str::<#name>(s).ok())
                        .unwrap_or_else(|| <#name as peace_rpc_error::TonicError>::tonic_error(status)),
                    _ => <#name as peace_rpc_error::TonicError>::tonic_error(status),
                }
            }
        }

        impl std::convert::From<#name> for tonic::Status {
            fn from(e: #name) -> Self {
                let mut status = tonic::Status::internal(format!("peace service error: {}", e));

                status.metadata_mut().insert(
                    peace_rpc_error::SERVICE_ERROR_HEADER,
                    serde_json::to_string(&e)
                        .unwrap_or("could not serialize: {e}".to_string())
                        .parse()
                        .unwrap_or(tonic::metadata::MetadataValue::from_static("unable to create metadata value"))
                );
                status
            }
        }
    };
    gen.into()
}
