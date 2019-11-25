use crate::{VersionGroupResolver, VersionGroupURI};
use failure::Fail;
use std::collections::HashMap;

/// Maps the version number for each deserialization type name
pub trait VersionMap: Sync + for<'a> VersionMapIter<'a> {
    fn get(&self, type_id: &str) -> Option<usize>;
}
pub trait VersionMapIter<'a> {
    type Iter: Iterator<Item = (&'a str, usize)>;
    fn iter(&'a self) -> Self::Iter;
}
pub type DefaultVersionMap<'a> = HashMap<&'a str, usize>;

#[derive(Debug, Fail)]
#[fail(
    display = "Both version uri {} and {} contains the type {}",
    uri_1, uri_2, type_name
)]
pub struct TypeInMultipleVersionGroups {
    type_name: String,
    uri_1: VersionGroupURI<'static>,
    uri_2: VersionGroupURI<'static>,
}

#[derive(Debug, Fail)]
#[fail(display = "Unknown version uri {}", uri)]
pub struct UnknownVersionURI {
    uri: VersionGroupURI<'static>,
}

#[derive(Debug, Fail)]
pub enum AggregateVersionMapError {
    #[fail(display = "{}", _0)]
    UnknownVersionURI(UnknownVersionURI),
    #[fail(display = "{}", _0)]
    TypeInMultipleVersionGroups(TypeInMultipleVersionGroups),
}

/// Build a new `VersionMap` that contains all the information for
/// all provided version uris
pub fn aggregate_version_maps<'a, VMR: VersionGroupResolver>(
    uris: &[VersionGroupURI<'a>],
    resolver: &VMR,
) -> Result<HashMap<String, usize>, AggregateVersionMapError>
where
    VMR::VM: VersionMap,
{
    let mut result = HashMap::new();
    for (i, uri) in uris.iter().enumerate() {
        if let Some(version_map) = resolver.resolve(uri) {
            for (k, v) in version_map.iter() {
                if let Some(_) = result.get(k) {
                    // We found two version maps requesting a version
                    // of the same type.
                    // Version map must have disjoint keys.
                    //
                    // This should happens during development, so we take
                    // time here to find exactly where this type is defined
                    // to have the appropriate error.

                    for j in 0..(i - 1) {
                        // unwrap: We know previous uri have a version_map
                        let version_map = resolver.resolve(&uris[j]).unwrap();
                        if let Some(_) = version_map.get(k) {
                            // We found the other version map
                            return Err(AggregateVersionMapError::TypeInMultipleVersionGroups(
                                TypeInMultipleVersionGroups {
                                    type_name: k.to_string(),
                                    uri_1: uri.to_static(),
                                    uri_2: uris[i].to_static(),
                                },
                            ));
                        }
                    }
                }

                result.insert(k.to_string(), v);
            }
        } else {
            return Err(AggregateVersionMapError::UnknownVersionURI(
                UnknownVersionURI {
                    uri: uri.to_static(),
                },
            ));
        }
    }
    Ok(result)
}
