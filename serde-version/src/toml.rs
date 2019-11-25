use crate::version_map::aggregate_version_maps;
use crate::{
    AggregateVersionMapError, DeserializeVersioned, VersionGroupResolver, VersionGroupURIs,
    VersionedDeserializer,
};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", 0)]
    Toml(::toml::de::Deserializer::Error),
    #[fail(display = "{}", .)]
    AggregateError(AggregateVersionMapError),
}

pub fn deserialize<'de, T: DeserializeVersioned<'de>>(
    input: &str,
    resolver: &VersionGroupResolver,
) -> Result<T, ::toml::de::Deserializer::Error> {
    let mut de = ::toml::de::Deserializer::new(input);
    let uris: VersionGroupURIs = serde::Deserialize::deserialize(&mut de)?;

    let version_map = aggregate_version_maps(&uris, resolver)?;

    // unwrap: the VersionGroupURIs ends with an array, so the token ']'
    // must exists as we successfully deserialized it.
    let end_of_version_header = input.find(']').unwrap();

    let input_left = input.split_at(end_of_version_header + 1).1;
    let mut de2 = ::toml::de::Deserializer::new(input_left);
    let mut de3 = VersionedDeserializer::new(&mut de2, &version_map);
    DeserializeVersioned::deserialize_versioned(&mut de3, &version_map)
}
