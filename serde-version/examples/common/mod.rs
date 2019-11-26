use serde_version::{DeserializeVersioned, VersionMap, VersionedDeserializer};
use std::fmt::Debug;

pub fn deserialize_test<
    'de,
    T: DeserializeVersioned<'de, VM> + PartialEq + Debug,
    VM: VersionMap,
>(
    input: &'de str,
    v: T,
    version_map: VM,
) {
    let mut ron_deserializer = ron::de::Deserializer::from_str(input).unwrap();
    let deserializer = VersionedDeserializer::new(&mut ron_deserializer, version_map.clone());
    let de = T::deserialize_versioned(deserializer, version_map).unwrap();

    assert_eq!(v, de);
}
