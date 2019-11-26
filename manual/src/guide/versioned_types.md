# Versioned types

In order to deserialize properly, `serde-version` expect a [`VersionMap`] with a version number
for each deserialized types. (If a version is not defined, then the default deserialization occurs).

You can use the macros [`version_map_new!`] and [`version_map_static!`] to help you create [`VersionMap`].
 

[`VersionMap`]: .
[`version_map_new!`]: .
[`version_map_static!`]: .