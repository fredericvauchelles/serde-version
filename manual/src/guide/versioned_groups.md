# Versioned groups

## Motivation

Types barely change alone, usually a set of types will change during the release of a software.
So keeping track of a single version number that will define the schema to use for a set of types.

## Version group and uris

The version group is identified by a uri (`VersionGroupURI`).
This uri is defined by an api group and a version, like `"api_group:version"`

You can also define an `enum` that maps to the appropriate uri with [`version_group_enum`].

## Resolving the version groups

When deserializing with serde-version, you need to provide a way to find the version group for 
each version uri defined in the version header.

This is the purpose of the `VersionGroupResolver`.

You can define one statically with the macro `version_group_resolver_static`.

See the `versioned_group` example.

## Example in Toml
```toml
# Version header with 2 version uris
v = ["org.my.company:1.0.2", "org.my.plugin:1.3.2"] 

[config]
name = "my config name"

[my_plugin]
plugin_name = "plugin name"
```

[`version_group_enum`]: .