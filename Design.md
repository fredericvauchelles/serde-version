# Design

## Versioned deserialization

A type can be composed of multiple other types and each of those can be versioned.
Thus we need to know how to migrate any types encountered during the deserialization.

In the serde model, there 4 case were we actually want to migrate the type:
1. when deserializing an element in a sequence
1. when deserializing a key of a map
1. when deserializing a value of a map
1. when deserializing a variant of an enum

So, we introduce a new trait `DeserializeVersioned` to handle those cases.
`DeserializeVersioned::next_element`, `DeserializeVersioned::next_value`, `DeserializeVersioned::next_key`, `DeserializeVersioned::variant` are the functions that will handles thoses case.

The trait have a default implementation which is the serde implementation without versioning.

To have the actual implementation, we use a derive macro that will implement those functions as a specialization.

## Versioned groups

During software development, we barely version a single type, usually a set of types are versioned together.
It makes simpler to declare the versioned used of a file.

So instead of declaring the version number of each type, we can instead define a version group id and deduce the 
version number of a set of types.