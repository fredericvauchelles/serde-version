# Serde version &emsp; [![Build Status]][travis] [![Latest Version]][crates.io]

[Build Status]: https://travis-ci.org/fredpointzero/serde-version.svg?branch=master
[travis]: https://travis-ci.org/fredpointzero/serde-version
[Latest Version]: https://img.shields.io/crates/v/serde-version.svg
[crates.io]: https://crates.io/crates/serde-version
# Serde version

Versioning support for serde.

When software are developped and used at the same time the data formats may change
from one version to another and persisting data may be produced by a specific version
and loaded by another version.

Serde version provide a versioning feature for serde for the main use cases.

See the guide [here](https://fredpointzero.github.io/crates/serde-version/).

Note 1: Requires the specialization feature.
Note 2: Use the `derive` feature to generate the `DeserializeVersioned` implementation

## Goals of Serde version

We aim at solving the case were a type or a set of types in a deserializer's
data needs to be upgraded to their latest format.
This is the case when a mandatory property was added or removed,
or an existing property changed.

## Non goals

This is based on types that can be upgraded individually.
Types that needs to be upgraded together is way more complex to handle
and usually relies on domain specific deserializer.

So, these data format should be handle with specific `Deserialize` traits implementations.
