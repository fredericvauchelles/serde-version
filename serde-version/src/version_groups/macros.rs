/// Generate a static field initialized with specified version groups
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_version;
/// #
/// # struct A;
/// # struct B;
/// version_group_resolver_static! {
///     VERSIONS = {
///         ( "my.api_group", "1.0.0" ) => { A => 2, B => 3, },
///     }
/// }
/// # // To have extern crate syntax
/// # fn main() {
/// # }
/// ```
#[macro_export]
macro_rules! version_group_resolver_static {
    ($(#[$attr:meta])* $id:ident = { $($body:tt)* }) => {
        __version_group_resolver_static! { ($(#[$attr])*), (), ($id), ($($body)*) }
    };
    ($(#[$attr:meta])* pub $id:ident = { $($body:tt)* }) => {
        __version_group_resolver_static! { ($(#[$attr])*), (pub), ($id), ($($body)*) }
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) $id:ident = { $($body:tt)* }) => {
        __version_group_resolver_static! { ($(#[$attr])*), (pub ($($vis)+)), ($id), ($($body)*) }
    };
}

#[macro_export]
macro_rules! __version_group_resolver_static {
    (
        ($(#[$attr:meta])*), ($($vis:tt)*), ($id:ident),
        ($(($api_group:expr, $api_version:expr) => { $($path:path => $version:expr),*, }),*,)
    ) => {
        lazy_static! {
            $(#[$attr])* $($vis)* static ref $id: $crate::DefaultVersionGroupResolver<'static>
                = version_group_resolver_new! { $(
                    ($api_group, $api_version) => { $($path => $version),*, }
                ),*, };
        }
    };
}

/// Generates a static field initialized with specified `VersionMap`.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_version;
/// #
/// # struct A;
/// # struct B;
/// version_map_static! {
///     TEST_1 = { A => 2, B => 3, }
/// }
/// # // To have extern crate syntax
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! version_map_static {
    ($(#[$attr:meta])* $id:ident = { $($body:tt)* }) => {
        __version_map_static!(($(#[$attr:meta])*), (), ($id), ($($body)*));
    };
    ($(#[$attr:meta])* pub $id:ident = { $($body:tt)* }) => {
        __version_map_static!(($(#[$attr:meta])*), (pub), ($id), ($($body)*));
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) $id:ident = { $($body:tt)* }) => {
        __version_map_static!(($(#[$attr:meta])*), (pub ($($vis)+)), ($id), ($($body)*));
    };
}

#[macro_export]
macro_rules! __version_map_static {
    (($(#[$attr:meta])*), ($($vis:tt)*), ($id:ident), ($($path:path => $version:expr),*,)) => {
        lazy_static! {
            $(#[$attr])* $($vis)* static ref $id: $crate::DefaultVersionMap<'static>
                = version_map_new!{ $($path => $version),*, };
        }
    }
}

/// Instantiate a `VersionGroupResolver` with specified version groups
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_version;
/// #
/// # struct A;
/// # struct B;
/// # // To have extern crate syntax
/// # fn main() {
/// let resolver = version_group_resolver_new! {
///     ( "my.api_group", "1.0.0" ) => { A => 2, B => 3, },
/// };
/// # }
/// ```
#[macro_export]
macro_rules! version_group_resolver_new {
    ($(($api_group:expr, $api_version:expr) => { $($path:path => $version:expr),*, }),*,) => {
        {
            let vec: Vec<((&str, &str), Box<$crate::DefaultVersionMap>)> =
            vec![
            $((
                ($api_group, $api_version),
                Box::new(version_map_new!{ $($path => $version),*, })
            )),*,];
            vec.into_iter()
                .collect::<$crate::DefaultVersionGroupResolver<'_>>()
        }
    };
}

/// Instantiate a `VersionMap` with specified type's version
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_version;
/// #
/// # struct A;
/// # struct B;
/// # // To have extern crate syntax
/// # fn main() {
/// let resolver = version_map_new! {
///     A => 2,
///     B => 3,
/// };
/// # }
/// ```
#[macro_export]
macro_rules! version_map_new {
    ($($path:path => $version:expr),*,) => {
        {
            vec![
                $(($crate::exports::get_type_name::<$path>(), $version)),*,
            ]
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>()
        }
    };
}

/// Create an enum that maps an entry to a `VersionGroupURI<T>`
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_version;
/// # #[macro_use]
/// # extern crate serde;
/// # use serde::{Serialize, Deserialize};
/// #
/// version_group_enum! {
///     #[derive(Serialize, Deserialize)]
///     enum Versions {
///         V1 as "v1" => "my.api_group:1.0.0",
///         V2 as "v2" => "my.second.api_group:1.2.0",
///     }
/// }
/// # // To have extern crate syntax
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! version_group_enum {
    ($(#[$attr:meta])* enum $id:ident { $($body:tt)* }) => {
        __version_group_enum! { ($(#[$attr])*), (), ($id), ($($body)*) }
    };
    ($(#[$attr:meta])* pub enum $id:ident { $($body:tt)* }) => {
        __version_group_enum! { ($(#[$attr])*), (pub), ($id), ($($body)*) }
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) enum $id:ident { $($body:tt)* }) => {
        __version_group_enum! { ($(#[$attr])*), (pub ($($vis)+)), ($id), ($($body)*) }
    };
}

#[macro_export]
macro_rules! __version_group_enum {
    (($(#[$attr:meta])*), ($($vis:tt)*), ($id:ident), ($($entry:ident as $alias:expr => $uri:expr),*,)) => {
        $(#[$attr])* $($vis)* enum $id {
            $(#[serde(alias = $alias)] $entry),*
        }

        impl ::std::convert::From<$id> for $crate::VersionGroupURI<'static> {
            fn from(v: $id) -> Self {
                use ::std::convert::TryInto;
                match v {$(
                    $id::$entry => $uri.try_into().unwrap()
                ),*,}
            }
        }

        impl ::std::convert::From<$id> for &'static $crate::VersionGroupURI<'static> {
            fn from(v: $id) -> Self {
                use ::std::convert::TryInto;

                $(
                    lazy_static! {
                        static ref $entry: $crate::VersionGroupURI<'static> =
                            $uri.try_into().unwrap();
                    }
                )*

                match v {$(
                    $id::$entry => &*$entry
                ),*,}
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{DefaultVersionMap, VersionGroupURI};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::convert::TryFrom;

    struct A;
    struct B;

    #[test]
    fn version_map_new_works() {
        let version_map = version_map_new! {
            A => 1,
            B => 2,
        };

        assert_eq!(
            vec![
                ("serde_version::version_groups::macros::tests::A", 1),
                ("serde_version::version_groups::macros::tests::B", 2),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>(),
            version_map
        );
    }

    version_map_static! {
        TEST_1 = { A => 2, B => 3, }
    }

    #[test]
    fn version_map_static_works() {
        assert_eq!(
            &vec![
                ("serde_version::version_groups::macros::tests::A", 2),
                ("serde_version::version_groups::macros::tests::B", 3),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>(),
            &*TEST_1
        );
    }

    version_group_enum! {
        #[derive(Serialize, Deserialize)]
        enum Versions {
            V1 as "v1" => "my.api_group:1.0.0",
            V2 as "v2" => "my.second.api_group:1.2.0",
        }
    }

    #[test]
    fn version_group_enum_works() {
        assert_eq!(
            VersionGroupURI::try_from("my.api_group:1.0.0").unwrap(),
            Versions::V1.into()
        );
        assert_eq!(
            VersionGroupURI::try_from("my.second.api_group:1.2.0").unwrap(),
            Versions::V2.into()
        );
    }

    #[test]
    fn version_group_resolver_new_works() {
        let resolver = version_group_resolver_new! {
            ( "my.api_group", "1.0.0" ) => { A => 1, B => 2, },
        };

        assert_eq!(
            resolver
                .into_iter()
                .map(|(k, v)| (k, unsafe {
                    Box::from_raw(Box::into_raw(v) as *mut DefaultVersionMap)
                }))
                .collect::<HashMap<_, _>>(),
            {
                let vec = vec![(
                    ("my.api_group", "1.0.0"),
                    Box::new(version_map_new! { A => 1, B => 2, }),
                )];
                vec.into_iter().collect::<HashMap<_, _>>()
            }
        );
    }

    version_group_resolver_static! {
        VERSIONS = {
            ( "my.api_group", "1.0.0" ) => { A => 2, B => 3, },
        }
    }
}
