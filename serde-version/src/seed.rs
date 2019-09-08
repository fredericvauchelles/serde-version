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

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        self.seed
            .deserialize(VersionedDeserializer::new(deserializer, self.version_map))
            .map_err(|err| err.into_error())
    }
}
