use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemStruct};

#[proc_macro_attribute]
pub fn convert(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let item = parse_macro_input!(item as ItemStruct);

    if args.len() != 1 {
        return syn::Error::new_spanned(item, "Only one parameter.")
            .to_compile_error()
            .into();
    }
    let arg = &args[0];
    let type_arg = match arg {
        syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
            path.get_ident().unwrap().clone()
        },
        _ => {
            panic!("Invalid argument")
        },
    };

    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) =
        item.generics.split_for_impl();
    let fields = item
        .fields
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    let expanded = quote! {
        #item
        impl #impl_generics ::std::convert::From<#type_arg #ty_generics> for #name #ty_generics #where_clause {
            fn from(other: #type_arg #ty_generics) -> Self {
                #name {
                    #(#fields: other.#fields),*
                }
            }
        }

        impl #impl_generics ::std::convert::Into<#type_arg #ty_generics> for #name #ty_generics #where_clause {
            fn into(self) -> #type_arg #ty_generics {
                #type_arg {
                    #(#fields: self.#fields),*
                }
            }
        }
    };

    expanded.into()
}
