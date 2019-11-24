# Unsupported Serde feature with versioning

## `deserialize_in_place` is not supported

Deserializing in place with versioning support is way more complicated,
so we don't deal with this in this crate.

## Not supported with `deserialize_with` callback

You must take care of the versioning in your callback

## Versioning is only supported for structs and enums

There is no use case where versioning tuples and the unit type is useful.
