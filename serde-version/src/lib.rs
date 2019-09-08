//! # Serde version
//!
//! Versioning support for serde.
//!
//! When software are developped and used at the same time the data formats may change
//! from one version to another and persisting data may be produced by a specific version
//! and loaded by another version.
//!
//! Serde version provide a versioning feature for serde for the main use cases.
//!
//! Note 1: Requires the specialization feature.
//! Note 2: Use the `derive` feature to generate the `DeserializeVersioned` implementation
//!
//! ## Goals of Serde version
//!
//! We aim at solving the case were a type or a set of types in a deserializer's
//! data needs to be upgraded to their latest format.
//! This is the case when a mandatory property was added or removed,
//! or an existing property changed.
//!
//! Note: There already is support for added optional properties in serde.
//! (Use the `default` feature of serde)
//!
//! Example:
//! Let's have a file containing these structure with those version number:
//! `A: 1, B: 1, C: 2` and the current version numbers are: `A: 3, B: 2, C: 4`.
//!
//! Then in latest code version, we have the former data structures versions,
//! let's call them: `Av1`, `Av2`, `Bv1`, `Cv1`, `Cv2`, `Cv3`.
//!
//! Deserializing, whenever a structure `A`, `B` or `C` is ran into,
//! then it is loaded with the appropriate format (in our case it will be `Av1`, `Bv1` and `Cv2`)
//! and then converted to `A`, `B` or `C` using the From trait.
//!
//! ## Non goals
//!
//! This is based on types that can be upgraded individually.
//! Types that needs to be upgraded together is way more complex to handle
//! and usually relies on domain specific deserializer.
//!
//! So, these data format should be handle with specific `Deserialize` traits implementations.
//!
//! # Unsupported Serde feature with versioning
//!
//! ## `deserialize_in_place` is not supported
//!
//! Deserializing in place with versioning support is way more complicated,
//! so we don't deal with this in this crate.
//!
//! ## Not supported with `deserialize_with` callback
//!
//! You must take care of the versioning in your callback
//!
//! ## Versioning is only supported for structs and enums
//!
//! There is no use case where versioning tuples and the unit type is useful.
//!
//! # Usage
//!
//! To describe the previous versions of a type, we use the `#[versions(...)]` attribute along with
//! the `DeserializeVersioned` trait.
//!
//! Authoring example:
//! ```dont_compile
//! // Version 1 of struct A
//! // It must implement Deserialize, so it can be loaded by serde
//! #[derive(Deserialize)]
//! // It must be identified by A during deserialization
//! #[serde(rename = "A")]
//! struct Av1 {
//!   a: u8
//! }
//!
//! // Current version of struct A
//! // It must implement Deserialize and DeserializeVersioned
//! #[derive(Deserialize, DeserializeVersioned)]
//! // We use the versions attribute to define the previous versions
//! #[versions("Av1")]
//! // So, Version n°1 of A is Av1, Versions n°2 (current) of A is A
//! struct A {
//!   // We moved a property
//!   b: u8
//! }
//!
//! // A must implement From for all previous type, so we implement From<Av1>
//! impl From<Av1> for A {
//!   fn from(v: Av1) -> Self {
//!     Self {
//!       b: v.a
//!     }
//!   }
//! }
//! ```
//!
//! To perform the deserialization with the versioning support, we need to do two steps:
//! 1. Get the `VersionMap` which holds the version number to use per type
//! 1. Call the `deserialize_versioned` method with the `VersionMap`
//!
//! Note: The id used to find the version number of a type during deserialization is
//!   the deserialization name of the type.
//!
//! Execution example:
//! ```rust
//! # #![feature(specialization)]
//! #
//! # #[macro_use]
//! # extern crate serde_version_derive;
//! #
//! # use serde::Deserialize;
//! # use serde_version::DeserializeVersioned;
//! # use std::fmt::Debug;
//! #
//! # #[derive(Deserialize)]
//! # #[serde(rename = "A")]
//! # struct Av1 {
//! #   a: u8
//! # }
//! # #[derive(Deserialize, DeserializeVersioned, PartialEq, Debug)]
//! # #[versions("Av1")]
//! # struct A {
//! #   b: u8
//! # }
//! # impl From<Av1> for A {
//! #   fn from(v: Av1) -> Self {
//! #     Self {
//! #       b: v.a
//! #     }
//! #   }
//! # }
//!
//! #[derive(Deserialize, PartialEq, Debug)]
//! struct AInMap {
//!   a: A,
//! }
//!
//! fn main() {
//!   // Use ron as data format for this example
//!   use ron;
//!   use serde_version::DeserializeVersioned;
//!
//!   // First get a header
//!   // Here, we use the version 1 of `A`
//!   let versions: serde_version::VersionMap = ron::de::from_str(r#"{ "A": 1 }"#).unwrap();
//!   
//!   // Let's deserialize some values
//!   
//!   // Deserialize directly A
//!   let mut deserializer = ron::de::Deserializer::from_str(r#"A(a: 1)"#).unwrap();
//!   let value = A::deserialize_versioned(&mut deserializer, &versions).unwrap();
//!   assert_eq!(value, A { b: 1 });
//!   
//!   // Deserialize A contained in a struct property
//!   let mut deserializer = ron::de::Deserializer::from_str(r#"AInMap(a: A(a: 2))"#).unwrap();
//!   // Note: All types implementing `Deserialize` will also implement `DeserializeVersioned`
//!   let value = AInMap::deserialize_versioned(&mut deserializer, &versions).unwrap();
//!   assert_eq!(value.a, A { b: 2});
//! }
//! ```
//!
//! ## `VersionedDeserializer`
//!
//! Under the hood, `deserialize_version` wraps the provided deserializer with
//! the `VersionedDeserializer` to support the versioning.

#![feature(specialization)]

// Re-export #[derive(Serialize, Deserialize)].
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "serde_version_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate serde_version_derive;
#[cfg(feature = "serde_version_derive")]
#[doc(hidden)]
pub use serde_version_derive::*;
#[macro_use]
extern crate failure;

pub use deserializer::{VersionMap, VersionedDeserializer};
use serde::de::{EnumAccess, MapAccess, SeqAccess};
use std::fmt::Display;

#[derive(Debug, Hash, PartialEq, Eq, Fail)]
#[fail(display = "Invalid version {} for {}", version, type_id)]
pub struct InvalidVersionError {
    pub version: usize,
    pub type_id: String,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Error<E> {
    DeserializeError(E),
    InvalidVersionError(InvalidVersionError),
    Custom(String),
}

impl<E> Error<E>
where
    E: serde::de::Error,
{
    pub fn into_error(self) -> E {
        match self {
            Error::Custom(err) => serde::de::Error::custom(err),
            Error::DeserializeError(err) => err,
            Error::InvalidVersionError(err) => serde::de::Error::custom(format!("{}", err)),
        }
    }
}

impl<E> Error<Error<E>>
where
    E: serde::de::Error,
{
    pub fn reduce(self) -> Error<E> {
        match self {
            Error::Custom(err) | Error::DeserializeError(Error::Custom(err)) => Error::Custom(err),
            Error::InvalidVersionError(err)
            | Error::DeserializeError(Error::InvalidVersionError(err)) => {
                Error::InvalidVersionError(err)
            }
            Error::DeserializeError(Error::DeserializeError(err)) => Error::DeserializeError(err),
        }
    }
}

impl<E> std::fmt::Display for Error<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::DeserializeError(ref e) => write!(f, "{}", e),
            Error::InvalidVersionError(ref e) => {
                write!(f, "Unknown version {} for type {}", e.version, e.type_id)
            }
            Error::Custom(ref e) => write!(f, "{}", e),
        }
    }
}

impl<E> std::error::Error for Error<E> where E: std::error::Error {}

impl<E> serde::de::Error for Error<E>
where
    E: serde::de::Error,
{
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(format!("{}", msg))
    }
}

/// Trait for versioning support during deserialization
///
/// Use the `derive` feature to generate the implementation from `#[derive(DeserializeVersioned)]`
/// and `#[versions(...)]` attribute.
pub trait DeserializeVersioned<'de>: serde::Deserialize<'de> {
    /// Entry point for the versioned deserialization
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn deserialize_versioned<D>(
        deserializer: D,
        _version_map: &'de VersionMap,
    ) -> Result<Self, Error<D::Error>>
    where
        D: serde::de::Deserializer<'de>,
    {
        Self::deserialize(deserializer).map_err(Error::DeserializeError)
    }

    /// Entry point for deserializing an element in a sequence
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    #[inline]
    fn next_element<S>(
        seq_access: &mut S,
        _version_map: &'de VersionMap,
    ) -> Result<Option<Self>, Error<S::Error>>
    where
        S: SeqAccess<'de>,
    {
        seq_access
            .next_element_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }

    /// Entry point for deserializing the next map value
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    #[inline]
    fn next_value<M>(
        map_access: &mut M,
        _version_map: &'de VersionMap,
    ) -> Result<Self, Error<M::Error>>
    where
        M: MapAccess<'de>,
    {
        map_access
            .next_value_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }

    /// Entry point for deserializing the next key value
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    #[inline]
    fn next_key<M>(
        map_access: &mut M,
        _version_map: &'de VersionMap,
    ) -> Result<Option<Self>, Error<M::Error>>
    where
        M: MapAccess<'de>,
    {
        map_access
            .next_key_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }

    /// Entry point for deserializing the next variant
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    #[inline]
    fn variant<E>(
        enum_access: E,
        _version_map: &'de VersionMap,
    ) -> Result<(Self, E::Variant), Error<E::Error>>
    where
        E: EnumAccess<'de>,
    {
        enum_access
            .variant_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }
}

impl<'de, T: serde::Deserialize<'de>> DeserializeVersioned<'de> for T {
    default fn deserialize_versioned<D>(
        deserializer: D,
        version_map: &'de VersionMap,
    ) -> Result<Self, Error<D::Error>>
    where
        D: serde::de::Deserializer<'de>,
    {
        let version_deserializer = VersionedDeserializer::new(deserializer, version_map);
        T::deserialize(version_deserializer)
    }
}

mod deserializer {
    use super::visitor::VersionedVisitor;
    use super::Error;
    use serde::Deserializer;
    use std::collections::HashMap;

    /// Maps the version number for each deserialization type name
    pub type VersionMap = HashMap<String, usize>;

    /// A wrapper around a deserialize to support the deserialization.
    ///
    /// This deserializer will wrap all calls where specialization is required. (Like
    /// `next_element`, `next_value`, ...)
    pub struct VersionedDeserializer<'de, D>
    where
        D: Deserializer<'de>,
    {
        deserializer: D,
        version_map: &'de VersionMap,
        marker: std::marker::PhantomData<&'de usize>,
    }

    impl<'de, D> VersionedDeserializer<'de, D>
    where
        D: Deserializer<'de>,
    {
        pub fn new(deserializer: D, version_map: &'de VersionMap) -> Self {
            Self {
                deserializer,
                version_map,
                marker: std::marker::PhantomData,
            }
        }
    }

    macro_rules! forward_deserialize {
    ($name:ident) => {forward_deserialize!($name, );};
    ($name:ident, $($arg:tt => $ty:ty),*) => {
        fn $name<V>(self, $($arg: $ty,)* visitor: V) -> Result<V::Value, Error<D::Error>>
            where V: serde::de::Visitor<'de>
        {
            let visitor = VersionedVisitor::new(
                visitor,
                self.version_map,
            );
            self.deserializer.$name($($arg,)* visitor).map_err(Error::DeserializeError)
        }
    }
}

    impl<'de, D: Deserializer<'de>> Deserializer<'de> for VersionedDeserializer<'de, D> {
        type Error = Error<D::Error>;

        forward_deserialize!(deserialize_any);
        forward_deserialize!(deserialize_bool);
        forward_deserialize!(deserialize_u8);
        forward_deserialize!(deserialize_u16);
        forward_deserialize!(deserialize_u32);
        forward_deserialize!(deserialize_u64);
        forward_deserialize!(deserialize_i8);
        forward_deserialize!(deserialize_i16);
        forward_deserialize!(deserialize_i32);
        forward_deserialize!(deserialize_i64);
        forward_deserialize!(deserialize_f32);
        forward_deserialize!(deserialize_f64);
        forward_deserialize!(deserialize_char);
        forward_deserialize!(deserialize_str);
        forward_deserialize!(deserialize_string);
        forward_deserialize!(deserialize_unit);
        forward_deserialize!(deserialize_option);
        forward_deserialize!(deserialize_seq);
        forward_deserialize!(deserialize_bytes);
        forward_deserialize!(deserialize_byte_buf);
        forward_deserialize!(deserialize_map);
        forward_deserialize!(deserialize_unit_struct, name => &'static str);
        forward_deserialize!(deserialize_newtype_struct, name => &'static str);
        forward_deserialize!(deserialize_tuple_struct, name => &'static str, len => usize);
        forward_deserialize!(deserialize_struct,
                         name => &'static str,
                         fields => &'static [&'static str]);
        forward_deserialize!(deserialize_identifier);
        forward_deserialize!(deserialize_tuple, len => usize);
        forward_deserialize!(deserialize_enum,
                         name => &'static str,
                         variants => &'static [&'static str]);
        forward_deserialize!(deserialize_ignored_any);
    }
}

mod visitor {
    use super::Error;
    use super::{VersionMap, VersionedDeserializer};
    use crate::seed::VersionedSeed;
    use crate::DeserializeVersioned;
    use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
    use serde::{Deserialize, Deserializer};

    pub struct VersionedVisitor<'v, V> {
        visitor: V,
        version_map: &'v VersionMap,
    }

    impl<'v, V> VersionedVisitor<'v, V> {
        pub fn new(visitor: V, version_map: &'v VersionMap) -> Self {
            Self {
                visitor,
                version_map,
            }
        }
    }

    macro_rules! forward_visit {
        ($name:ident, $ty:ty) => {
            #[inline]
            fn $name<E>(self, v: $ty) -> Result<V::Value, E>
                where E: serde::de::Error
            {
                self.visitor.$name(v)
            }
        }
    }

    impl<'de, V> Visitor<'de> for VersionedVisitor<'de, V>
    where
        V: Visitor<'de>,
    {
        type Value = V::Value;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.visitor.expecting(formatter)
        }

        forward_visit!(visit_bool, bool);
        forward_visit!(visit_i8, i8);
        forward_visit!(visit_i16, i16);
        forward_visit!(visit_i32, i32);
        forward_visit!(visit_i64, i64);
        forward_visit!(visit_u8, u8);
        forward_visit!(visit_u16, u16);
        forward_visit!(visit_u32, u32);
        forward_visit!(visit_u64, u64);
        forward_visit!(visit_f32, f32);
        forward_visit!(visit_f64, f64);
        forward_visit!(visit_char, char);
        forward_visit!(visit_bytes, &[u8]);
        forward_visit!(visit_byte_buf, Vec<u8>);
        forward_visit!(visit_str, &str);
        forward_visit!(visit_string, String);
        forward_visit!(visit_borrowed_str, &'de str);

        #[inline]
        fn visit_unit<E>(self) -> Result<V::Value, E>
        where
            E: serde::de::Error,
        {
            self.visitor.visit_unit()
        }

        #[inline]
        fn visit_none<E>(self) -> Result<V::Value, E>
        where
            E: serde::de::Error,
        {
            self.visitor.visit_none()
        }

        #[inline]
        fn visit_some<D>(self, deserializer: D) -> Result<V::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let deserializer = VersionedDeserializer::new(deserializer, self.version_map);
            self.visitor
                .visit_some(deserializer)
                .map_err(|err| err.into_error())
        }

        #[inline]
        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<V::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let deserializer = VersionedDeserializer::new(deserializer, self.version_map);
            self.visitor
                .visit_newtype_struct(deserializer)
                .map_err(|err| err.into_error())
        }

        #[inline]
        fn visit_seq<V2>(self, visitor: V2) -> Result<V::Value, V2::Error>
        where
            V2: SeqAccess<'de>,
        {
            let visitor = VersionedVisitor {
                visitor,
                version_map: self.version_map,
            };
            self.visitor
                .visit_seq(visitor)
                .map_err(|err| err.into_error())
        }

        #[inline]
        fn visit_map<V2>(self, visitor: V2) -> Result<V::Value, V2::Error>
        where
            V2: MapAccess<'de>,
        {
            let visitor = VersionedVisitor {
                visitor,
                version_map: self.version_map,
            };
            self.visitor
                .visit_map(visitor)
                .map_err(|err| err.into_error())
        }

        #[inline]
        fn visit_enum<V2>(self, visitor: V2) -> Result<V::Value, V2::Error>
        where
            V2: EnumAccess<'de>,
        {
            let visitor = VersionedVisitor {
                visitor,
                version_map: self.version_map,
            };
            self.visitor
                .visit_enum(visitor)
                .map_err(|err| err.into_error())
        }
    }

    impl<'de, V> SeqAccess<'de> for VersionedVisitor<'de, V>
    where
        V: SeqAccess<'de>,
    {
        type Error = Error<V::Error>;

        #[inline]
        fn next_element_seed<T>(
            &mut self,
            seed: T,
        ) -> Result<Option<<T as DeserializeSeed<'de>>::Value>, Self::Error>
        where
            T: DeserializeSeed<'de>,
        {
            let seed = VersionedSeed::new(seed, self.version_map);
            self.visitor
                .next_element_seed(seed)
                .map_err(Error::DeserializeError)
        }

        #[inline]
        fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
        where
            T: Deserialize<'de>,
        {
            <T as DeserializeVersioned<'de>>::next_element(self, self.version_map)
                .map_err(|err| err.reduce())
        }
    }

    impl<'de, V> MapAccess<'de> for VersionedVisitor<'de, V>
    where
        V: MapAccess<'de>,
    {
        type Error = Error<V::Error>;

        #[inline]
        fn next_key_seed<K>(
            &mut self,
            seed: K,
        ) -> Result<Option<<K as DeserializeSeed<'de>>::Value>, Self::Error>
        where
            K: DeserializeSeed<'de>,
        {
            let seed = VersionedSeed::new(seed, self.version_map);
            self.visitor
                .next_key_seed(seed)
                .map_err(Error::DeserializeError)
        }

        #[inline]
        fn next_value_seed<S>(
            &mut self,
            seed: S,
        ) -> Result<<S as DeserializeSeed<'de>>::Value, Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            let seed = VersionedSeed::new(seed, self.version_map);
            self.visitor
                .next_value_seed(seed)
                .map_err(Error::DeserializeError)
        }

        #[inline]
        #[allow(clippy::type_complexity)]
        fn next_entry_seed<K, V2>(
            &mut self,
            kseed: K,
            vseed: V2,
        ) -> Result<Option<(K::Value, V2::Value)>, Self::Error>
        where
            K: DeserializeSeed<'de>,
            V2: DeserializeSeed<'de>,
        {
            let kseed = VersionedSeed::new(kseed, self.version_map);
            let vseed = VersionedSeed::new(vseed, self.version_map);
            self.visitor
                .next_entry_seed(kseed, vseed)
                .map_err(Error::DeserializeError)
        }

        #[inline]
        fn next_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where
            K: Deserialize<'de>,
        {
            <K as DeserializeVersioned<'de>>::next_key(self, self.version_map)
                .map_err(|err| err.reduce())
        }

        #[inline]
        fn next_value<V2>(&mut self) -> Result<V2, Self::Error>
        where
            V2: Deserialize<'de>,
        {
            <V2 as DeserializeVersioned<'de>>::next_value(self, self.version_map)
                .map_err(|err| err.reduce())
        }

        fn size_hint(&self) -> Option<usize> {
            self.visitor.size_hint()
        }
    }

    impl<'de, V> EnumAccess<'de> for VersionedVisitor<'de, V>
    where
        V: EnumAccess<'de>,
    {
        type Error = Error<V::Error>;
        type Variant = VersionedVisitor<'de, V::Variant>;

        #[inline]
        #[allow(clippy::type_complexity)]
        fn variant_seed<S>(
            self,
            seed: S,
        ) -> Result<(S::Value, VersionedVisitor<'de, V::Variant>), Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            let seed = VersionedSeed::new(seed, self.version_map);
            match self.visitor.variant_seed(seed) {
                Ok((value, variant)) => {
                    let variant = VersionedVisitor {
                        visitor: variant,
                        version_map: self.version_map,
                    };
                    Ok((value, variant))
                }
                Err(e) => Err(Error::DeserializeError(e)),
            }
        }

        #[inline]
        fn variant<V2>(self) -> Result<(V2, Self::Variant), Self::Error>
        where
            V2: Deserialize<'de>,
        {
            let version_map = self.version_map;
            <V2 as DeserializeVersioned<'de>>::variant(self, version_map)
                .map_err(|err| err.reduce())
        }
    }

    impl<'de, V> VariantAccess<'de> for VersionedVisitor<'de, V>
    where
        V: VariantAccess<'de>,
    {
        type Error = Error<V::Error>;

        #[inline]
        fn unit_variant(self) -> Result<(), Self::Error> {
            self.visitor.unit_variant().map_err(Error::DeserializeError)
        }

        #[inline]
        fn newtype_variant_seed<S>(self, seed: S) -> Result<S::Value, Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            let seed = VersionedSeed::new(seed, self.version_map);
            self.visitor
                .newtype_variant_seed(seed)
                .map_err(Error::DeserializeError)
        }

        #[inline]
        fn tuple_variant<V2>(self, len: usize, visitor: V2) -> Result<V2::Value, Self::Error>
        where
            V2: Visitor<'de>,
        {
            let visitor = VersionedVisitor {
                visitor,
                version_map: self.version_map,
            };
            self.visitor
                .tuple_variant(len, visitor)
                .map_err(Error::DeserializeError)
        }

        #[inline]
        fn struct_variant<V2>(
            self,
            fields: &'static [&'static str],
            visitor: V2,
        ) -> Result<V2::Value, Self::Error>
        where
            V2: Visitor<'de>,
        {
            let visitor = VersionedVisitor {
                visitor,
                version_map: self.version_map,
            };
            self.visitor
                .struct_variant(fields, visitor)
                .map_err(Error::DeserializeError)
        }
    }
}

mod seed {
    use super::VersionMap;
    use crate::VersionedDeserializer;
    use serde::de::DeserializeSeed;
    use serde::Deserializer;

    pub struct VersionedSeed<'v, S> {
        seed: S,
        version_map: &'v VersionMap,
    }

    impl<'v, S> VersionedSeed<'v, S> {
        pub fn new(seed: S, version_map: &'v VersionMap) -> Self {
            Self { seed, version_map }
        }
    }

    impl<'de, 'v, S> DeserializeSeed<'de> for VersionedSeed<'de, S>
    where
        S: DeserializeSeed<'de>,
    {
        type Value = S::Value;

        fn deserialize<D>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            self.seed
                .deserialize(VersionedDeserializer::new(deserializer, self.version_map))
                .map_err(|err| err.into_error())
        }
    }
}
