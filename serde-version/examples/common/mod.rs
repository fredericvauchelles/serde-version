use serde_version::{DeserializeVersioned, VersionMap, VersionedDeserializer};
use std::fmt::Debug;

pub fn deserialize_test<'de, T: DeserializeVersioned<'de> + PartialEq + Debug>(
    input: &'de str,
    v: T,
    version_map: &'de dyn VersionMap,
) {
    let mut ron_deserializer = ron::de::Deserializer::from_str(input).unwrap();
    let deserializer = VersionedDeserializer::new(&mut ron_deserializer, version_map);
    let de = T::deserialize_versioned(deserializer, version_map).unwrap();

    assert_eq!(v, de);
}
