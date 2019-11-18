use crate::{VersionMap, VersionedDeserializer};
use serde::de::DeserializeSeed;
use serde::Deserializer;

/// Seed wrapper:
///
/// Wrap calls to deserialize with a VersionedDeserializer
pub struct VersionedSeed<'v, S, VM> {
    seed: S,
    version_map: &'v VM,
}

impl<'v, S, VM> VersionedSeed<'v, S, VM> {
    pub fn new(seed: S, version_map: &'v VM) -> Self {
        Self { seed, version_map }
    }
}

impl<'de, 'v, S, VM> DeserializeSeed<'de> for VersionedSeed<'de, S, VM>
where
    S: DeserializeSeed<'de>,
    VM: VersionMap,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        self.seed
            .deserialize(VersionedDeserializer::new(deserializer, self.version_map))
            .map_err(|err| err.into_error())
    }
}
