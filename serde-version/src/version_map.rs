use std::collections::HashMap;

pub use aggregate::{
    AggregateVersionMap, AggregateVersionMapError, TypeInMultipleVersionGroups, UnknownVersionURI,
};

/// Maps the version number for each deserialization type name
pub trait VersionMap: Clone + Sync + for<'a> VersionMapIter<'a> {
    fn get(&self, type_id: &str) -> Option<usize>;
}
/// Has an iter method
pub trait VersionMapIter<'a> {
    type Iter: Iterator<Item = (&'a str, usize)>;
    fn iter(&'a self) -> Self::Iter;
}
pub type DefaultVersionMap<'a> = HashMap<&'a str, usize>;

mod aggregate {
    use crate::version_map::VersionMapIter;
    use crate::{VersionGroupResolver, VersionGroupURI, VersionMap};
    use std::collections::HashMap;

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

    /// Implement this trait to provide a method to aggregate version maps
    ///
    /// The default implementation will create a new instance at each call.
    /// But you can override this behaviour to provide a cache system.
    pub trait AggregateVersionMap {
        /// Build a new `VersionMap` that contains all the information for
        /// all provided version uris
        fn aggregate_version_maps<'a, VMR: VersionGroupResolver>(
            &self,
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
                                    return Err(
                                        AggregateVersionMapError::TypeInMultipleVersionGroups(
                                            TypeInMultipleVersionGroups {
                                                type_name: k.to_string(),
                                                uri_1: uri.to_static(),
                                                uri_2: uris[i].to_static(),
                                            },
                                        ),
                                    );
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
    }
    impl AggregateVersionMap for () {}
}

mod version_map_impls {
    use crate::version_map::VersionMapIter;
    use crate::VersionMap;
    use std::borrow::Borrow;
    use std::collections::HashMap;
    use std::hash::{BuildHasher, Hash};

    impl<T: Borrow<str> + Hash + Eq + Sync + Clone + 'static, S: BuildHasher + Sync + Clone>
        VersionMap for HashMap<T, usize, S>
    {
        fn get(&self, type_id: &str) -> Option<usize> {
            std::collections::HashMap::get(self, type_id).cloned()
        }
    }
    impl<'i, T: Borrow<str> + Hash + Eq + 'i, S: BuildHasher + Sync> VersionMapIter<'i>
        for HashMap<T, usize, S>
    {
        type Iter = std::iter::Map<
            std::collections::hash_map::Iter<'i, T, usize>,
            fn((&'i T, &'i usize)) -> (&'i str, usize),
        >;

        fn iter(&'i self) -> Self::Iter {
            HashMap::<T, usize, S>::iter(self).map(|(k, v)| (k.borrow(), *v))
        }
    }

    impl<'a, T: VersionMap> VersionMap for &'a T
    where
        &'a T: Clone,
    {
        fn get(&self, type_id: &str) -> Option<usize> {
            <T as VersionMap>::get(self, type_id)
        }
    }
    impl<'a, 'i, T: VersionMapIter<'i>> VersionMapIter<'i> for &'a T {
        type Iter = <T as VersionMapIter<'i>>::Iter;

        fn iter(&'i self) -> Self::Iter {
            <T as VersionMapIter<'i>>::iter(self)
        }
    }

    impl<'a, T: VersionMap> VersionMap for &'a mut T
    where
        &'a mut T: Clone,
    {
        fn get(&self, type_id: &str) -> Option<usize> {
            <T as VersionMap>::get(self, type_id)
        }
    }
    impl<'a, 'i, T: VersionMapIter<'i>> VersionMapIter<'i> for &'a mut T {
        type Iter = <T as VersionMapIter<'i>>::Iter;

        fn iter(&'i self) -> Self::Iter {
            <T as VersionMapIter<'i>>::iter(self)
        }
    }
}
