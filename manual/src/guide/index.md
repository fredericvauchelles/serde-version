# Guide

The way serde-version works is pretty simple.

The best practice is to have a _version header_ which contains enough data to know
which version to use for each type to deserialize.

There are several strategies to handle this situation.

At the lowest level, serde-version expect to have a [`VersionMap`] with a version number
for each deserialized type. See [Versioned Types] or the `versioned_types` example.

But you can also choose to define a unique version for a set of types. We name this a
_version group_ defined by a _version uri_. See [Versioned Groups] or the `versioned_groups` example.

[Versioned Groups]: ./versioned_groups.md
[Versioned Types]: ./versioned_types.md
[`VersionMap`]: .