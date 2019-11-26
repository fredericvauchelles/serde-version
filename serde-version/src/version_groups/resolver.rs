use super::VersionGroupURI;
use crate::DefaultVersionMap;
use std::borrow::Borrow;
use std::collections::HashMap;

/// Find the `VersionMap` associated to a `VersionGroupURI`
pub trait VersionGroupResolver {
    /// The type of the `VersionMap` returned
    type VM;

    /// Find the `VersionMap` associated to a `VersionGroupURI`
    fn resolve<'s, 'u: 's, T: Borrow<VersionGroupURI<'u>> + 's>(
        &'s self,
        uri: &'u T,
    ) -> Option<&Self::VM>;
}

pub type DefaultVersionGroupResolver<'a> = HashMap<(&'a str, &'a str), Box<DefaultVersionMap<'a>>>;

impl<'a> VersionGroupResolver for DefaultVersionGroupResolver<'a> {
    type VM = DefaultVersionMap<'a>;

    fn resolve<'s, 'u: 's, T: Borrow<VersionGroupURI<'u>> + 's>(
        &'s self,
        uri: &'u T,
    ) -> Option<&Self::VM> {
        let uri = uri.borrow();
        self.get(&(uri.api_group(), uri.version()))
            .map(std::ops::Deref::deref)
    }
}
