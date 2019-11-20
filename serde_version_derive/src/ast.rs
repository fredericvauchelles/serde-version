use proc_macro_util::prelude::Ctxt;

pub struct Container<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Container,
    pub generics: &'a syn::Generics,
}

pub mod symbols {
    use proc_macro_util::prelude::Symbol;

    pub const DEFAULT: Symbol = Symbol("default");
    pub const INDEX: Symbol = Symbol("index");
    pub const SELF: Symbol = Symbol("self");
    pub const TYPE: Symbol = Symbol("type");
    pub const VERSIONS: Symbol = Symbol("versions");
    pub const VERSION: Symbol = Symbol("version");
    pub const VERSION_SHORTHAND: Symbol = Symbol("v");
}

pub mod attr {
    use super::symbols::{DEFAULT, TYPE, VERSION, VERSIONS};
    use ast::symbols::{INDEX, SELF, VERSION_SHORTHAND};
    use proc_macro_util::prelude::{Attr, Ctxt};
    use quote::ToTokens;
    use std::collections::HashMap;
    use syn::{Meta, NestedMeta};
    use util::get_serde_version_meta_items;

    pub struct Container {
        versions: Option<Versions>,
    }

    impl Container {
        pub fn from_ast(cx: &Ctxt, item: &syn::DeriveInput) -> Self {
            let mut versions = Attr::none(cx, VERSIONS);

            let mut self_version_defined = false;

            match item.data {
                syn::Data::Struct(syn::DataStruct {
                    ref struct_token, ..
                }) => {
                    let mut error_message = None;
                    let mut parsed_versions = HashMap::new();

                    for meta_items in item.attrs.iter().filter_map(get_serde_version_meta_items) {
                        for nested in meta_items.iter() {
                            match *nested {
                                // Parse 'version(index = 1, type = "typeA", default)'
                                // Parse 'v(index = 1, type = "typeA", default)'
                                NestedMeta::Meta(Meta::List(ref list))
                                    if list.path == VERSION || list.path == VERSION_SHORTHAND =>
                                {
                                    let mut path = None;
                                    let mut is_self = false;
                                    let mut default = false;
                                    let mut index = None;

                                    for item in &list.nested {
                                        match item {
                                            NestedMeta::Meta(Meta::NameValue(ref pair))
                                                if pair.path == TYPE =>
                                            {
                                                match pair.lit {
                                                    syn::Lit::Str(ref str) => {
                                                        if let Ok(path2) = str.parse::<syn::Path>()
                                                        {
                                                            path = Some(path2);
                                                        }
                                                    }
                                                    _ => {
                                                        error_message = Some(format!("'type' expect a string value, received {}", pair.lit.clone().into_token_stream().to_string()));
                                                        break;
                                                    }
                                                };
                                            }
                                            NestedMeta::Meta(Meta::NameValue(ref pair))
                                                if pair.path == INDEX =>
                                            {
                                                match pair.lit {
                                                    syn::Lit::Int(ref int) => {
                                                        if let Ok(value) = int.base10_parse() {
                                                            index = Some(value);
                                                        }
                                                    }
                                                    _ => {
                                                        error_message = Some(format!("'index' expect an integer value, received {}", pair.lit.clone().into_token_stream().to_string()));
                                                        break;
                                                    }
                                                };
                                            }
                                            NestedMeta::Meta(Meta::Path(ref p)) if p == DEFAULT => {
                                                default = true;
                                            }
                                            NestedMeta::Meta(Meta::Path(ref p)) if p == SELF => {
                                                is_self = true;
                                                self_version_defined = true;
                                            }
                                            value => {
                                                error_message = Some(format!(
                                                    "unknown attribute {:?}",
                                                    value.into_token_stream().to_string()
                                                ));
                                                break;
                                            }
                                        }
                                    }

                                    error_message = error_message
                                        .or_else(|| {
                                            // type and self are exclusive
                                            if path.is_some() && is_self {
                                                Some(
                                                    "'type' and 'self' can't be defined together."
                                                        .to_string(),
                                                )
                                            }
                                            // type or self is mandatory
                                            else if !path.is_some() && !is_self {
                                                Some(
                                                    "One of 'type' or 'self' must be defined."
                                                        .to_string(),
                                                )
                                            } else {
                                                None
                                            }
                                        })
                                        .or_else(|| {
                                            if index.is_some() {
                                                None
                                            } else {
                                                Some("'index' is required.".to_string())
                                            }
                                        });

                                    if error_message.is_some() {
                                        break;
                                    }

                                    parsed_versions.insert(
                                        index.unwrap(),
                                        Version {
                                            path: path
                                                .map(PathOrSelf::Path)
                                                .unwrap_or(PathOrSelf::SelfType),
                                            index: index.unwrap(),
                                            is_default: default,
                                        },
                                    );
                                }
                                ref value => {
                                    error_message = Some(format!(
                                        "unknown attribute {:?}",
                                        value.into_token_stream().to_string()
                                    ));
                                    break;
                                }
                            }
                        }
                    }

                    error_message = error_message.or_else(|| {
                        if self_version_defined {
                            versions.set(
                                &item,
                                Versions {
                                    versions: parsed_versions,
                                },
                            );
                            None
                        } else {
                            Some("A version must be defined for 'self'.".to_string())
                        }
                    });

                    if let Some(error_message) = error_message {
                        cx.error_spanned_by(
                            struct_token,
                            format!("Error while parsing the attribute: {}.", error_message),
                        );
                    }
                }
                syn::Data::Enum(syn::DataEnum { ref enum_token, .. }) => {
                    cx.error_spanned_by(enum_token, "#[versions(...)] can only be used on structs");
                }
                syn::Data::Union(syn::DataUnion {
                    ref union_token, ..
                }) => {
                    cx.error_spanned_by(
                        union_token,
                        "#[versions(...)] can only be used on structs",
                    );
                }
            }

            Container {
                versions: versions.get(),
            }
        }

        pub fn versions(&self) -> Option<&Versions> {
            self.versions.as_ref()
        }
    }

    pub enum PathOrSelf {
        SelfType,
        Path(syn::Path),
    }
    pub struct Versions {
        versions: HashMap<usize, Version>,
    }
    pub struct Version {
        pub path: PathOrSelf,
        pub index: usize,
        pub is_default: bool,
    }
    impl std::ops::Deref for Versions {
        type Target = HashMap<usize, Version>;

        fn deref(&self) -> &Self::Target {
            &self.versions
        }
    }
}

impl<'a> Container<'a> {
    pub fn from_ast(cx: &Ctxt, item: &'a syn::DeriveInput) -> Container<'a> {
        let attrs = attr::Container::from_ast(cx, item);

        Container {
            ident: item.ident.clone(),
            attrs,
            generics: &item.generics,
        }
    }
}

#[cfg(test)]
mod tests {
    use ast::attr::PathOrSelf;
    use ast::Container;
    use proc_macro_util::prelude::Ctxt;
    use quote::ToTokens;
    use std::collections::HashMap;

    #[test]
    fn parse_container() {
        let item: proc_macro2::TokenStream = quote! {
            #[versions(v(index = 1, type = "Av1"), version(index = 3, type = "namespace::Av2", default), v(index = 4, self))]
            struct A<T> { marker: std::marker::PhantomData<T>, }
        };

        let item = syn::parse2::<syn::DeriveInput>(item).unwrap();

        let cx = Ctxt::new();
        let cont = Container::from_ast(&cx, &item);
        cx.check().unwrap();

        assert_eq!("A".to_owned(), cont.ident.to_string());
        assert_eq!(
            "< T >".to_owned(),
            cont.generics.to_token_stream().to_string()
        );
        assert!(cont.attrs.versions().is_some());
        assert_eq!(
            std::ops::Deref::deref(cont.attrs.versions().unwrap()).len(),
            3
        );
        assert_eq!(
            vec![
                (1, ("Av1".to_owned(), false)),
                (3, ("namespace :: Av2".to_owned(), true)),
                (4, ("<self>".to_owned(), false))
            ]
            .into_iter()
            .collect::<HashMap<_, _>>(),
            cont.attrs
                .versions()
                .unwrap()
                .iter()
                .map(|(&i, v)| (
                    i,
                    (
                        match &v.path {
                            PathOrSelf::SelfType => "<self>".to_string(),
                            PathOrSelf::Path(path) => path.to_token_stream().to_string(),
                        },
                        v.is_default
                    )
                ))
                .collect::<HashMap<_, _>>()
        );
    }
}
