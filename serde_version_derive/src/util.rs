// From serde

use proc_macro2::{Ident, Span, TokenStream};
use syn::Meta;

pub fn get_serde_version_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path == crate::ast::symbols::VERSIONS {
        match attr.parse_meta() {
            Ok(Meta::List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}

pub fn wrap_in_const(
    serde_path: Option<&syn::Path>,
    serde_version_path: Option<&syn::Path>,
    trait_: &str,
    ty: &Ident,
    code: TokenStream,
) -> TokenStream {
    fn use_(path: Option<&syn::Path>, source: &syn::Path, alias: &syn::Path) -> TokenStream {
        match path {
            Some(path) => quote! {
                use #path as #alias;
            },
            None => quote! {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]
                extern crate #source as #alias;
            },
        }
    }

    let dummy_const = Ident::new(&format!("_IMPL_{}_FOR_{}", trait_, ty), Span::call_site());

    let use_serde = use_(
        serde_path,
        &syn::parse2::<syn::Path>(quote! { serde }).unwrap(),
        &syn::parse2::<syn::Path>(quote! { _serde }).unwrap(),
    );
    let use_serde_version = use_(
        serde_version_path,
        &syn::parse2::<syn::Path>(quote! { serde_version }).unwrap(),
        &syn::parse2::<syn::Path>(quote! { _serde_version }).unwrap(),
    );

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #use_serde
            #use_serde_version
            #code
        };
    }
}
