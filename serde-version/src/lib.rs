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
//! See the guide [here](https://fredpointzero.github.io/crates/serde-version/).
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
//! ## Non goals
//!
//! This is based on types that can be upgraded individually.
//! Types that needs to be upgraded together is way more complex to handle
//! and usually relies on domain specific deserializer.
//!
//! So, these data format should be handle with specific `Deserialize` traits implementations.

// Some doc test needs external crates
// In that case, we need the main function
#![allow(clippy::needless_doctest_main)]
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

// Rexport lazy static
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;
#[doc(hidden)]
pub use lazy_static::*;

#[macro_use]
extern crate failure;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod deserializer;
mod seed;
mod version_groups;
mod version_map;
mod visitor;

#[macro_use]
mod utils;

pub mod exports;
#[cfg(feature = "toml-support")]
pub mod toml;

pub use deserializer::VersionedDeserializer;
use serde::de::{EnumAccess, MapAccess, SeqAccess};
use std::fmt::Display;
pub use version_groups::{
    DefaultVersionGroupResolver, VersionGroupResolver, VersionGroupURI, VersionGroupURIs,
};
pub use version_map::{
    AggregateVersionMap, AggregateVersionMapError, DefaultVersionMap, TypeInMultipleVersionGroups,
    UnknownVersionURI, VersionMap,
};

/// Error used when a provided version number is not handled by current code
#[derive(Debug, Hash, PartialEq, Eq, Fail)]
#[fail(display = "Unknown version {} for type {}", version, type_id)]
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
pub trait DeserializeVersioned<'de, VM: VersionMap>: serde::Deserialize<'de> {
    /// Entry point for the versioned deserialization
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    ///
    /// Note: The `VM` type should be a reference type for better performance.
    ///   The version_map is cloned during the deserialization process, but cloning a reference
    ///   is cheap.
    fn deserialize_versioned<D>(deserializer: D, _version_map: VM) -> Result<Self, Error<D::Error>>
    where
        D: serde::de::Deserializer<'de>;

    /// Entry point for deserializing an element in a sequence
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn next_element<S>(
        seq_access: &mut S,
        version_map: VM,
    ) -> Result<Option<Self>, Error<S::Error>>
    where
        S: SeqAccess<'de>;

    /// Entry point for deserializing the next map value
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn next_value<M>(map_access: &mut M, _version_map: VM) -> Result<Self, Error<M::Error>>
    where
        M: MapAccess<'de>;

    /// Entry point for deserializing the next key value
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn next_key<M>(map_access: &mut M, _version_map: VM) -> Result<Option<Self>, Error<M::Error>>
    where
        M: MapAccess<'de>;

    /// Entry point for deserializing the next variant
    ///
    /// Implement this method to specialize the deserialization for a particular type.
    ///
    /// The default implementation ignore the versioning
    fn variant<E>(enum_access: E, _version_map: VM) -> Result<(Self, E::Variant), Error<E::Error>>
    where
        E: EnumAccess<'de>;
}

impl<'de, T: serde::Deserialize<'de>, VM: VersionMap> DeserializeVersioned<'de, VM> for T {
    default fn deserialize_versioned<D>(
        deserializer: D,
        version_map: VM,
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
        _version_map: VM,
    ) -> Result<Option<Self>, Error<S::Error>>
    where
        S: SeqAccess<'de>,
    {
        seq_access
            .next_element_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    default fn next_value<M>(map_access: &mut M, _version_map: VM) -> Result<Self, Error<M::Error>>
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
        _version_map: VM,
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
        _version_map: VM,
    ) -> Result<(Self, E::Variant), Error<E::Error>>
    where
        E: EnumAccess<'de>,
    {
        enum_access
            .variant_seed(std::marker::PhantomData)
            .map_err(Error::DeserializeError)
    }
}
