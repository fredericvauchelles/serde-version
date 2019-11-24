//! Example of versioning
//!
//! Here we have one type `A` with 2 revision `Av1` and `Av2`.
//! This shows how to build a version map and use it during the deserialization
//! to choose the appropriate version.
//!
#![feature(specialization)]

extern crate serde;
#[macro_use]
extern crate serde_version_derive;

use serde::Deserialize;
use serde_version::{DefaultVersionMap, DeserializeVersioned, VersionMap, VersionedDeserializer};
use std::fmt::Debug;

#[derive(Deserialize)]
#[serde(rename(deserialize = "A"))]
struct Av1 {
    a: u8,
}

#[derive(Deserialize)]
#[serde(rename(deserialize = "A"))]
struct Av2 {
    b: u8,
}

#[derive(Deserialize, PartialEq, DeserializeVersioned, Debug)]
#[serde(rename(deserialize = "A"))]
#[versions(
    v(index = 1, type = "Av1"),
    v(index = 3, type = "Av2"),
    v(index = 4, self)
)]
struct A {
    c: u8,
}

impl From<Av1> for A {
    fn from(v: Av1) -> Self {
        Self { c: v.a }
    }
}
impl From<Av2> for A {
    fn from(v: Av2) -> Self {
        Self { c: v.b }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
struct ContainsA {
    a: A,
}

fn deserialize_test<'de, T: DeserializeVersioned<'de> + PartialEq + Debug>(
    input: &'de str,
    v: T,
    version_map: &'de dyn VersionMap,
) {
    let mut ron_deserializer = ron::de::Deserializer::from_str(input).unwrap();
    let deserializer = VersionedDeserializer::new(&mut ron_deserializer, version_map);
    let de = T::deserialize_versioned(deserializer, version_map).unwrap();

    assert_eq!(v, de);
}

fn main() {
    let mut version_map = DefaultVersionMap::new();
    version_map.insert("A", 1);

    deserialize_test("A(a: 8)", A { c: 8 }, &version_map);
    deserialize_test(
        "ContainsA(a: A(a: 8))",
        ContainsA { a: A { c: 8 } },
        &version_map,
    );

    *version_map.get_mut("A").unwrap() = 3;
    deserialize_test("A(b: 8)", A { c: 8 }, &version_map);
    deserialize_test(
        "ContainsA(a: A(b: 8))",
        ContainsA { a: A { c: 8 } },
        &version_map,
    );

    *version_map.get_mut("A").unwrap() = 4;
    deserialize_test("A(c: 8))", A { c: 8 }, &version_map);
    deserialize_test(
        "ContainsA(a: A(c: 8))",
        ContainsA { a: A { c: 8 } },
        &version_map,
    );
}
