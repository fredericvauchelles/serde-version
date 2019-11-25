use crate::VersionGroupURI;
use failure::Fail;
use std::collections::HashMap;

/// Maps the version number for each deserialization type name
pub trait VersionMap: Sync {
    fn get(&self, type_id: &str) -> Option<usize>;
}
pub type DefaultVersionMap<'a> = HashMap<&'a str, usize>;

#[derive(Debug, Fail)]
#[fail(
    display = "Mismatching version detected for {}, \
               version required in {} is {} and in {} is {}",
    type_name, uri_1, version_1, uri_2, version_2
)]
pub struct MismatchingVersionsDetected {
    type_name: String,
    version_1: usize,
    uri_1: VersionGroupURI<'static>,
    version_2: usize,
    uri_2: VersionGroupURI<'static>,
}

pub fn aggregate_version_maps(
    input: &[&dyn VersionMap],
) -> Result<HashMap<String, usize>, MismatchingVersionsDetected> {
    let mut result = HashMap::new();
    for version_map in input {
        for (k, v) in version_map
    }
}
