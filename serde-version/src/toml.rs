use crate::{
    aggregate_version_maps, AggregateVersionMapError, DeserializeVersioned, Error,
    VersionGroupResolver, VersionGroupURIs, VersionMap, VersionedDeserializer,
};
use failure::Fail;
use serde::Serialize;

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

pub fn deserialize<'de, T: DeserializeVersioned<'de, VMR::VM>, VMR: VersionGroupResolver>(
    input: &'de str,
    resolver: &VMR,
) -> Result<T, DeserializeError>
where
    VMR::VM: VersionMap,
{
    let mut de = ::toml::de::Deserializer::new(input);
    let uris: VersionGroupURIs = serde::Deserialize::deserialize(&mut de)?;

    let version_map = aggregate_version_maps(&uris, resolver)?;

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

pub fn serialize_inplace<'s, T: Serialize>(
    str: &'s mut String,
    value: &T,
    uris: &VersionGroupURIs,
) -> Result<(), ::toml::ser::Error> {
    let mut ser = ::toml::ser::Serializer::new(str);

    Serialize::serialize(uris, &mut ser)?;
    Serialize::serialize(value, &mut ser)?;

    Ok(())
}

pub fn serialize<'s, T: Serialize>(
    value: &T,
    uris: &VersionGroupURIs,
) -> Result<String, ::toml::ser::Error> {
    let mut str = String::new();
    serialize_inplace(&mut str, value, uris)?;
    Ok(str)
}
