use crate::{DeserializeVersioned, VersionGroupResolver, VersionGroupURIs};
use failure::Fail;

#[derive(Debug, Fail)]
#[fail(display = "{}", .)]
pub enum Error {
    #[fail(display = "{}", 0)]
    Toml(::toml::de::Deserializer::Error),
    #[fail(display = "Unknown version uri: {}", uri)]
    UnknownVersionURI { uri: String },
}

pub fn deserialize<'de, T: DeserializeVersioned<'de>>(
    input: &str,
    resolver: &VersionGroupResolver,
) -> Result<T, ::toml::de::Deserializer::Error> {
    let mut de = ::toml::de::Deserializer::new(input);
    let versions: VersionGroupURIs = serde::Deserialize::deserialize(&mut de)?;

    let mut version_maps = Vec::with_capacity(versions.len());
    for uri in versions {
        let version_map = match resolver.resolve(uri) {
            Some(version_map) => version_map,
            None => {
                return Err(Error::UnknownVersionURI {
                    uri: uri.to_string(),
                })
            }
        };
        version_maps.push(version_map);
    }

    // unwrap: the VersionGroupURIs ends with an array, so the token ']'
    // must exists as we successfully deserialized it.
    let end_of_version_header = input.find(']').unwrap();

    let input_left = input.split_at(end_of_version_header + 1).1;
    let mut de2 = ::toml::de::Deserializer::new(input_left);
    DeserializeVersioned::deserialize_versioned(&mut de2, &version_maps)
}
