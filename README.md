# Serde version &emsp; [![Build Status]][travis] [![Latest Version]][crates.io]

[Build Status]: https://travis-ci.org/fredpointzero/serde-version.svg?branch=master
[travis]: https://travis-ci.org/fredpointzero/serde-version
[Latest Version]: https://img.shields.io/crates/v/serde-version.svg
[crates.io]: https://crates.io/crates/serde-version

## Versioning support for serde.

When software are developped and used at the same time the data formats may change
from one version to another and persisting data may be produced by a specific version
and loaded by another version.

Serde version provide a versioning feature for serde for the main use cases.

Note: requires the specialization feature.

## Goals of Serde version

We aim at solving the case were a type or a set of types in a deserializer's
data needs to be upgraded to their latest format.
This is the case when a mandatory property was added or removed,
or an existing property changed.

Note: There already is support for added optional properties in serde.
(Use the `default` feature of serde)

Example:
Let's have a file containing these structure with those version number:
`A: 1, B: 1, C: 2` and the current version numbers are: `A: 3, B: 2, C: 4`.

Then in latest code version, we have the former data structures versions,
let's call them: `Av1`, `Av2`, `Bv1`, `Cv1`, `Cv2`, `Cv3`.

Deserializing, whenever a structure `A`, `B` or `C` is ran into,
then it is loaded with the appropriate format (in our case it will be `Av1`, `Bv1` and `Cv2`)
and then converted to `A`, `B` or `C` using the From trait.

## Non goals

This is based on types that can be upgraded individually.
Types that needs to be upgraded together is way more complex to handle
and usually relies on domain specific deserializer.

So, these data format should be handle with specific `Deserialize` traits implementations.

# Unsupported Serde feature with versioning

## `deserialize_in_place` is not supported

Deserializing in place with versioning support is way more complicated,
so we don't deal with this in this crate.

## Not supported with `deserialize_with` callback

You must take care of the versioning in your callback

## Versioning is only supported for structs and enums

There is no use case where versioning tuples and the unit type is useful.

# Usage

To describe the previous versions of a type, we use the `#[versions(...)]` attribute along with
the `DeserializeVersioned` trait.

Authoring example:
```rust
// Version 1 of struct A
// It must implement Deserialize, so it can be loaded by serde
#[derive(Deserialize)]
// It must be identified by A during deserialization
#[serde(rename = "A")]
struct Av1 {
a: u8
}

// Current version of struct A
// It must implement Deserialize and DeserializeVersioned
#[derive(Deserialize, DeserializeVersioned)]
// We use the versions attribute to define the previous versions
#[versions("Av1")]
// So, Version n°1 of A is Av1, Versions n°2 (current) of A is A
struct A {
// We moved a property
b: u8
}

// A must implement From for all previous type, so we implement From<Av1>
impl From<Av1> for A {
fn from(v: Av1) -> Self {
 Self {
   b: v.a
 }
}
}
```

To perform the deserialization with the versioning support, we need to do two steps:
1. Get the `VersionMap` which holds the version number to use per type
1. Call the `deserialize_versioned` method with the `VersionMap`

Note: The id used to find the version number of a type during deserialization is
the deserialization name of the type.

Execution example:
```rust
#[derive(Deserialize)]
struct AInMap {
a: A,
}

fn main() {
// Use ron as data format for this example
use ron;
use serde_version::DeserializeVersioned;

// First get a header
// Here, we use the version 1 of `A`
let versions: serde_version::VersionMap = ron::de::from_str(r#"{ "A": 1 }"#).unwrap();
   
// Let's deserialize some values
   
// Deserialize directly A
let mut deserializer = ron::de::Deserializer::from_str(r#"A(a: 1)"#).unwrap();
let value = A::deserialize_versioned(&mut deserializer, &versions);
assert_eq!(value, A { b: 1 });
   
// Deserialize A contained in a struct property
let mut deserializer = ron::de::Deserializer::from_str(r#"AInMap(a: A(a: 2))"#);
// Note: All types implementing `Deserialize` will also implement `DeserializeVersioned`
let value = AInMap::deserialize_versioned(&mut deserializer, &versions);
assert_eq!(value.a, A { b: 2});
}
```
 
## `VersionedDeserializer`

Under the hood, `deserialize_version` wraps the provided deserializer with 
the `VersionedDeserializer` to support the versioning.