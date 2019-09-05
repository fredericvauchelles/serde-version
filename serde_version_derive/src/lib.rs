extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_macro_util;
extern crate syn;
#[macro_use]
extern crate quote;

mod ast;
mod de;
mod util;

#[proc_macro_derive(DeserializeVersioned, attributes(versions, serde))]
pub fn derive_deserialize_versioned(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse::<syn::DeriveInput>(input).unwrap();
    de::expand_derive_deserialize_versioned(&input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
