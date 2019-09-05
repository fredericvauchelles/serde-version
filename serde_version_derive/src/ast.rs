use proc_macro_util::prelude::Ctxt;

pub struct Container<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Container,
    pub generics: &'a syn::Generics,
}

pub mod symbols {
    use proc_macro_util::prelude::Symbol;

    pub const DEFAULT: Symbol = Symbol("default");
    pub const TYPE: Symbol = Symbol("type");
    pub const VERSIONS: Symbol = Symbol("versions");
    pub const VERSION: Symbol = Symbol("version");
}

pub mod attr {
    use super::symbols::{DEFAULT, TYPE, VERSION, VERSIONS};
    use proc_macro_util::prelude::{Attr, Ctxt};
    use syn::{Meta, NestedMeta};
    use util::get_serde_version_meta_items;

    pub struct Container {
        versions: Option<Versions>,
    }

    impl Container {
        pub fn from_ast(cx: &Ctxt, item: &syn::DeriveInput) -> Self {
            let mut versions = Attr::none(cx, VERSIONS);

            match item.data {
                syn::Data::Struct(syn::DataStruct {
                    ref struct_token, ..
                }) => {
                    let mut is_valid = true;
                    let mut parsed_versions = Vec::new();

                    for meta_items in item.attrs.iter().filter_map(get_serde_version_meta_items) {
                        for nested in meta_items.iter() {
                            match *nested {
                                // Parse "typeA" as shorthand for 'version(type = "typeA")'
                                NestedMeta::Lit(syn::Lit::Str(ref str)) => {
                                    if let Ok(path) = str.parse::<syn::Path>() {
                                        parsed_versions.push(Version {
                                            path,
                                            is_default: false,
                                        });
                                    } else {
                                        is_valid = false;
                                        break;
                                    }
                                }
                                // Parse 'version(type = "typeA", default)'
                                NestedMeta::Meta(Meta::List(ref list)) if list.path == VERSION => {
                                    let mut path = None;
                                    let mut default = false;
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
                                                        is_valid = false;
                                                        break;
                                                    }
                                                };
                                            }
                                            NestedMeta::Meta(Meta::Path(ref p)) if p == DEFAULT => {
                                                default = true;
                                            }
                                            _ => {
                                                is_valid = false;
                                                break;
                                            }
                                        }
                                    }
                                    // type path is mandatory
                                    is_valid &= path.is_some();
                                    if !is_valid {
                                        break;
                                    }

                                    parsed_versions.push(Version {
                                        path: path.unwrap(),
                                        is_default: default,
                                    })
                                }
                                _ => {
                                    is_valid = false;
                                    break;
                                }
                            }
                        }
                    }

                    if is_valid {
                        versions.set(
                            &item,
                            Versions {
                                versions: parsed_versions,
                            },
                        );
                    } else {
                        cx.error_spanned_by(
                            struct_token,
                            "malformed versions attribute, expected `versions(\"type1\", version(type = \"type2\", default), ...)`",
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

    pub struct Versions {
        versions: Vec<Version>,
    }
    pub struct Version {
        pub path: syn::Path,
        pub is_default: bool,
    }
    impl std::ops::Deref for Versions {
        type Target = [Version];

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
    use ast::Container;
    use proc_macro_util::prelude::Ctxt;
    use quote::ToTokens;

    #[test]
    fn parse_container() {
        let item: proc_macro2::TokenStream = quote! {
            #[versions("Av1", version(type = "namespace::Av2", default))]
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
            2
        );
        assert_eq!(
            vec![
                ("Av1".to_owned(), false),
                ("namespace :: Av2".to_owned(), true)
            ],
            cont.attrs
                .versions()
                .unwrap()
                .iter()
                .map(|v| (v.path.to_token_stream().to_string(), v.is_default))
                .collect::<Vec<_>>()
        );
    }
}
