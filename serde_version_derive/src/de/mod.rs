use ast::Container;
use proc_macro2::{Span, TokenStream};
use proc_macro_util::prelude::*;

pub fn expand_derive_deserialize_versioned(
    input: &syn::DeriveInput,
) -> Result<TokenStream, Vec<syn::Error>> {
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, input);
    ctxt.check()?;

    match cont.attrs.versions() {
        Some(versions) => {
            let de_impl_generics = {
                let mut generics = cont.generics.clone();
                generics.params = Some(syn::GenericParam::Lifetime(syn::LifetimeDef::new(
                    syn::Lifetime::new("'de", Span::call_site()),
                )))
                .into_iter()
                .chain(generics.params)
                .collect();
                generics
            };
            let ident = &cont.ident;
            let ty_generics = cont.generics;
            let where_clause = cont.generics.where_clause.as_ref();
            // TODO: Find the deserialization name from the serde attribute
            //   (like `#[serde(rename(deserialize = "deser_name"))]`
            let deser_name = syn::LitStr::new(&ident.to_string(), ident.span());

            let deserialize_arms = versions.iter()
                .enumerate()
                .map(|(i, version)| {
                    let version_number = i + 1;
                    let path = &version.path;
                    quote! {
                            Some(#version_number) => _serde::export::Result::map(
                                <#path as _serde_version::DeserializeVersioned>::deserialize_versioned(__deserializer, __version_map),
                                _serde::export::Into::into
                            ),
                        }
                })
                .collect::<Vec<_>>();
            let next_element_arms = versions.iter()
                .enumerate()
                .map(|(i, version)| {
                    let version_number = i + 1;
                    let path = &version.path;
                    quote! {
                            Some(#version_number) => _serde::export::Result::map(
                                <#path as _serde_version::DeserializeVersioned>::next_element(__seq_access, __version_map),
                                |v| _serde::export::Option::map(v, _serde::export::Into::into)
                            ),
                        }
                })
                .collect::<Vec<_>>();
            let next_value_arms= versions.iter()
                .enumerate()
                .map(|(i, version)| {
                    let version_number = i + 1;
                    let path = &version.path;
                    quote! {
                            Some(#version_number) => _serde::export::Result::map(
                                <#path as _serde_version::DeserializeVersioned>::next_value(__map_access, __version_map),
                                _serde::export::Into::into
                            ),
                        }
                })
                .collect::<Vec<_>>();
            let next_key_arms= versions.iter()
                .enumerate()
                .map(|(i, version)| {
                    let version_number = i + 1;
                    let path = &version.path;
                    quote! {
                            Some(#version_number) => _serde::export::Result::map(
                                <#path as _serde_version::DeserializeVersioned>::next_key(__map_access, __version_map),
                                |v| _serde::export::Option::map(v, _serde::export::Into::into)
                            ),
                        }
                })
                .collect::<Vec<_>>();
            let variant_arms= versions.iter()
                .enumerate()
                .map(|(i, version)| {
                    let version_number = i + 1;
                    let path = &version.path;
                    quote! {
                            Some(#version_number) => _serde::export::Result::map(
                                <#path as _serde_version::DeserializeVersioned>::variant(__enum_access, __version_map),
                                |(v, variant)| (_serde::export::Into::into(v), variant)
                            ),
                        }
                })
                .collect::<Vec<_>>();

            let code = quote! {
                impl #de_impl_generics _serde_version::DeserializeVersioned<'de> for #ident #ty_generics #where_clause {
                    fn deserialize_versioned<__D>(
                        __deserializer: __D,
                        __version_map: &'de _serde_version::VersionMap
                    ) -> _serde::export::Result<Self, _serde_version::Error<__D::Error>>
                    where
                        __D: _serde::Deserializer<'de>, {
                        match __version_map.get(#deser_name) {
                            #(#deserialize_arms)*
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(_serde_version::InvalidVersionError { version: v, type_id: #deser_name })),
                            None => <Self as _serde::Deserialize<'de>>::deserialize(__deserializer),
                        }
                    }

                    #[inline]
                    fn next_element<__S>(
                        __seq_access: &mut __S,
                        __version_map: &'de _serde_version::VersionMap,
                    ) -> _serde::export::Result<Option<Self>, _serde_version::Error<__S::Error>>
                    where
                        __S: _serde::de::SeqAccess<'de>
                    {
                        match __version_map.get(#deser_name) {
                            #(#next_element_arms)*
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(_serde_version::InvalidVersionError { version: v, type_id: #deser_name })),
                            None => <__S as _serde::de::SeqAccess<'de>>::next_element_seed(__seq_access, std::marker::PhantomData),
                        }
                    }

                    #[inline]
                    fn next_value<__M>(
                        __map_access: &mut __M,
                        __version_map: &'de _serde_version::VersionMap
                    ) -> _serde::export::Result<Self, _serde_version::Error<__M::Error>>
                    where
                        __M: _serde::de::MapAccess<'de>,
                    {
                        match __version_map.get(#deser_name) {
                            #(#next_value_arms)*
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(_serde_version::InvalidVersionError { version: v, type_id: #deser_name })),
                            None => <__M as _serde::de::MapAccess<'de>>::next_value_seed(__map_access, std::marker::PhantomData),
                        }
                    }

                    #[inline]
                    fn next_key<__M>(
                        __map_access: &mut __M,
                        __version_map: &'de _serde_version::VersionMap,
                    ) -> _serde::export::Result<Option<Self>, _serde_version::Error<__M::Error>>
                    where
                        __M: _serde::de::MapAccess<'de>,
                    {
                        match __version_map.get(#deser_name) {
                            #(#next_key_arms)*
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(_serde_version::InvalidVersionError { version: v, type_id: #deser_name })),
                            None => <__M as _serde::de::MapAccess<'de>>::next_key_seed(__map_access, std::marker::PhantomData),
                        }
                    }

                    #[inline]
                    fn variant<__E>(
                        __enum_access: __E,
                        __version_map: &'de _serde_version::VersionMap,
                    ) -> _serde::export::Result<(Self, __E::Variant), _serde_version::Error<__E::Error>>
                    where
                        __E: _serde::de::EnumAccess<'de>,
                    {
                        match __version_map.get(#deser_name) {
                            #(#variant_arms)*
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(_serde_version::InvalidVersionError { version: v, type_id: #deser_name })),
                            None => <__E as _serde::de::EnumAccess<'de>>::variant_seed(__enum_access, std::marker::PhantomData),
                        }
                    }
                }
            };
            Ok(crate::util::wrap_in_const(
                None,
                None,
                "DESERIALIZE_VERSIONED",
                ident,
                code,
            ))
        }
        None => Ok(TokenStream::new()),
    }
}
