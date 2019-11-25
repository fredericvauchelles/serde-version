//! Serialization utilities for the Toml format

use crate::version_map::AggregateVersionMap;
use crate::{
    AggregateVersionMapError, DeserializeVersioned, Error, VersionGroupResolver, VersionGroupURIs,
    VersionMap, VersionedDeserializer,
};
use failure::Fail;
use serde::Serialize;

/// Deserialization error
#[derive(Debug, Fail)]
pub enum DeserializeError {
    #[fail(display = "{}", 0)]
    Toml(::toml::de::Error),
    #[fail(display = "{}", 0)]
    De(Error<Error<::toml::de::Error>>),
    #[fail(display = "{}", 0)]
    AggregateError(AggregateVersionMapError),
}
impl_from_enum! {
    DeserializeError::Toml => ::toml::de::Error,
    DeserializeError::De => Error<Error<::toml::de::Error>>,
    DeserializeError::AggregateError => AggregateVersionMapError,
}

/// Deserialize a Toml string with versioning support
///
/// # Generic Parameters
/// - `T`: type to deserialize
/// - `VMR`: resolver to find the version groups to use
/// - `AGG`: aggregator to combine found version groups
///
/// # Parameters
/// - `input`: Toml formatted string, first entry is the version header
/// - `resolver`: resolver to find the version groups to use
/// - `aggregate`: aggregator to combine found version groups
///
/// # Returns
/// The deserialized value or the error that occurred
pub fn deserialize<'de, T, VMR, AGG>(
    input: &'de str,
    resolver: &VMR,
    aggregate: &AGG,
) -> Result<T, DeserializeError>
where
    VMR::VM: VersionMap,
    T: DeserializeVersioned<'de, VMR::VM>,
    VMR: VersionGroupResolver,
    AGG: AggregateVersionMap,
{
    let mut de = ::toml::de::Deserializer::new(input);
    let uris: VersionGroupURIs = serde::Deserialize::deserialize(&mut de)?;

    let version_map = aggregate.aggregate_version_maps(&uris, resolver)?;

    // unwrap: the VersionGroupURIs ends with an array, so the token ']'
    // must exists as we successfully deserialized it.
    let end_of_version_header = input.find(']').unwrap();

    let input_left = input.split_at(end_of_version_header + 1).1;
    let mut de2 = ::toml::de::Deserializer::new(input_left);
    let de3 = VersionedDeserializer::new(&mut de2, &version_map);
    Ok(DeserializeVersioned::deserialize_versioned(
        de3,
        &version_map,
    )?)
}

/// Serialize in a string a value formatted in Toml with its version header.
///
/// The version header will be serialized before the type.
///
/// # Generic Parameters
/// - `T`: type to serialize
///
/// # Parameters
/// - `str`: The string the will receive the data
/// - `value`: The value to serialize
/// - `uris`: The uris to serialize in the version header
///
/// # Returns
/// The error that occurred, if any.
pub fn serialize_inplace<'s, T: Serialize>(
    str: &'s mut String,
    value: &T,
    uris: &VersionGroupURIs,
) -> Result<(), ::toml::ser::Error> {
    let mut ser = ::toml::ser::Serializer::new(str);
    Serialize::serialize(&*uris, &mut ser)?;
    Serialize::serialize(value, &mut ser)?;

    Ok(())
}

/// Serialize a value formatted in Toml with its version header.
///
/// The version header will be serialized before the type.
///
/// # Generic Parameters
/// - `T`: type to serialize
///
/// # Parameters
/// - `value`: The value to serialize
/// - `uris`: The uris to serialize in the version header
///
/// # Returns
/// The formatted string or the error that occurred, if any.
pub fn serialize<'s, T: Serialize>(
    value: &T,
    uris: &VersionGroupURIs,
) -> Result<String, ::toml::ser::Error> {
    let mut str = String::new();
    serialize_inplace(&mut str, value, uris)?;
    Ok(str)
}
