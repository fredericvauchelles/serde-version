use super::VersionGroupURI;
use crate::VersionMap;
use std::collections::HashMap;

pub trait VersionGroupResolver {
    fn resolve<'s: 'o, 'u: 'o, 'o>(
        &'s self,
        uri: &'u VersionGroupURI<'u>,
    ) -> Option<&'o dyn VersionMap>;
}

pub type DefaultVersionGroupResolver<'a> = HashMap<(&'a str, &'a str), Box<dyn VersionMap>>;

impl<'a> VersionGroupResolver for DefaultVersionGroupResolver<'a> {
    fn resolve<'s: 'o, 'u: 'o, 'o>(
        &'s self,
        uri: &'u VersionGroupURI<'u>,
    ) -> Option<&'o dyn VersionMap> {
        self.get(&(uri.api_group(), uri.version()))
            .map(std::ops::Deref::deref)
    }
}
