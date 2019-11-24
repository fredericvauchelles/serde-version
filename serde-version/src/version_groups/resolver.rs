use super::VersionGroupURI;
use crate::VersionMap;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::FromIterator;

pub trait VersionGroupResolver {
    fn resolve<'s: 'o, 'u: 'o, 'o, T: Borrow<str>>(
        &'s self,
        uri: &'u VersionGroupURI<'u>,
    ) -> Option<&'o dyn VersionMap>;
}

pub struct DefaultVersionGroupResolver<'a>(HashMap<(&'a str, &'a str), Box<dyn VersionMap>>);

impl<'a> VersionGroupResolver for DefaultVersionGroupResolver<'a> {
    fn resolve<'s: 'o, 'u: 'o, 'o, T: Borrow<str>>(
        &'s self,
        uri: &'u VersionGroupURI<'u>,
    ) -> Option<&'o dyn VersionMap> {
        self.0
            .get(&(uri.api_group(), uri.version()))
            .map(std::ops::Deref::deref)
    }
}

impl<'a> FromIterator<((&'a str, &'a str), Box<dyn VersionMap>)>
    for DefaultVersionGroupResolver<'a>
{
    fn from_iter<T: IntoIterator<Item = ((&'a str, &'a str), Box<dyn VersionMap>)>>(
        iter: T,
    ) -> Self {
        Self(iter.into_iter().collect::<HashMap<_, _>>())
    }
}
