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
//! #[versions(v(index = 1, type = "Av1"), v(index = 2, type = "A"))]
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
//! # #[versions(v(index = 1, type = "Av1"), v(index = 2, self))]
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
//!   // Note: `rust_out` is the module used for the doc script
//!   let versions: serde_version::DefaultVersionMap = ron::de::from_str(r#"{ "rust_out::A": 1 }"#).unwrap();
//!   
//!   // Let's deserialize some values
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
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod deserializer;
pub mod exports;
mod seed;
mod version_groups;
mod visitor;

pub use deserializer::{DefaultVersionMap, VersionMap, VersionedDeserializer};
use serde::de::{EnumAccess, MapAccess, SeqAccess};
use std::fmt::Display;
pub use version_groups::{
    DefaultVersionGroupResolver, VersionGroupResolver, VersionGroupURI, VersionGroupURIs,
};

/// Error used when a provided version number is not handled by current code
#[derive(Debug, Hash, PartialEq, Eq, Fail)]
#[fail(display = "Invalid version {} for {}", version, type_id)]
pub struct InvalidVersionError {
    pub version: usize,
    pub type_id: String,
}

/// Error wrapper to add the version number related errors
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Error<E> {
    DeserializeError(E),
    InvalidVersionError(InvalidVersionError),
    Message(String),
}

impl<E> Error<E>
where
    E: serde::de::Error,
{
    pub fn into_error(self) -> E {
        match self {
            Error::Message(err) => serde::de::Error::custom(err),
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
            Error::Message(err) | Error::DeserializeError(Error::Message(err)) => {
                Error::Message(err)
            }
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
            Error::Message(ref e) => write!(f, "{}", e),
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
        Self::Message(format!("{}", msg))
    }
}

/// Trait for versioning support during deserialization
///
/// Use the `derive` feature to generate the implementation from `#[derive(DeserializeVersioned)]`
/// and `#[versions(...)]` attribute.
pub trait DeserializeVersioned<'de, VM: VersionMap = DefaultVersionMap>:
    serde::Deserialize<'de>
{
    /// Entry point for the versioned deserialization
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn deserialize_versioned<D>(
        deserializer: D,
        _version_map: &'de VM,
    ) -> Result<Self, Error<D::Error>>
    where
        D: serde::de::Deserializer<'de>;

    /// Entry point for deserializing an element in a sequence
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn next_element<S>(
        seq_access: &mut S,
        _version_map: &'de VM,
    ) -> Result<Option<Self>, Error<S::Error>>
    where
        S: SeqAccess<'de>;

    /// Entry point for deserializing the next map value
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn next_value<M>(map_access: &mut M, _version_map: &'de VM) -> Result<Self, Error<M::Error>>
    where
        M: MapAccess<'de>;

    /// Entry point for deserializing the next key value
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn next_key<M>(
        map_access: &mut M,
        _version_map: &'de VM,
    ) -> Result<Option<Self>, Error<M::Error>>
    where
        M: MapAccess<'de>;

    /// Entry point for deserializing the next variant
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn variant<E>(
        enum_access: E,
        _version_map: &'de VM,
    ) -> Result<(Self, E::Variant), Error<E::Error>>
    where
        E: EnumAccess<'de>;
}

impl<'de, VM: VersionMap, T: serde::Deserialize<'de>> DeserializeVersioned<'de, VM> for T {
    default fn deserialize_versioned<D>(
        deserializer: D,
        version_map: &'de VM,
    ) -> Result<Self, Error<D::Error>>
    where
        D: serde::de::Deserializer<'de>,
    {
        let version_deserializer = VersionedDeserializer::new(deserializer, version_map);
        T::deserialize(version_deserializer)
    }

    #[inline]
    default fn next_element<S>(
        seq_access: &mut S,
        _version_map: &'de VM,
    ) -> Result<Option<Self>, Error<S::Error>>
    where
        S: SeqAccess<'de>,
    {
        seq_access
            .next_element_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    default fn next_value<M>(
        map_access: &mut M,
        _version_map: &'de VM,
    ) -> Result<Self, Error<M::Error>>
    where
        M: MapAccess<'de>,
    {
        map_access
            .next_value_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    default fn next_key<M>(
        map_access: &mut M,
        _version_map: &'de VM,
    ) -> Result<Option<Self>, Error<M::Error>>
    where
        M: MapAccess<'de>,
    {
        map_access
            .next_key_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    default fn variant<E>(
        enum_access: E,
        _version_map: &'de VM,
    ) -> Result<(Self, E::Variant), Error<E::Error>>
    where
        E: EnumAccess<'de>,
    {
        enum_access
            .variant_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }
}
