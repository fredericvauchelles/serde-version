//! # Version groups
//!
//! Version groups are used to reference the version number of multiple types with a single
//! identifier.
//!

use crate::VersionMap;

/// Name of a version group
#[derive(Eq, PartialEq, Hash)]
pub struct GroupName(String);

/// Version id of a version group
#[derive(Eq, PartialEq, Hash)]
pub struct GroupVersion(String);

// TODO
//   Find a way to query version maps without using ctor
//   (may cause issue when statically linking libraries)
//   How to merge version maps?

pub fn version_map_for(name: &GroupName, version: &GroupVersion) -> Option<&VersionMap> {
    unimplemented!()
}

///
/// ```rust
/// version_group! {
///     "org.my_comp" {
///         "v1" {
///             "A" => 1
///         }
///     }
/// }
/// ```
///



