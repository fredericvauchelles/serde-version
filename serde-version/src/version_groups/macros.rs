/// Generate a static field initialized with specified version groups
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

macro_rules! __version_group_resolver_static {
    (
        ($(#[$attr:meta])*), ($($vis:tt)*), ($id:ident),
        ($(($api_group:expr, $api_version:expr) => { $($path:path => $version:expr),*, }),*,)
    ) => {
        lazy_static! {
            $(#[$attr])* $($vis)* static ref $id: DefaultVersionGroupResolver<'static>
                = version_group_resolver_new! { $(
                    ($api_group, $api_version) => { $($path => $version),*, }
                ),*, };
        }
    };
}

/// Generates a static field initialized with specified `VersionMap`.
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

macro_rules! __version_map_static {
    (($(#[$attr:meta])*), ($($vis:tt)*), ($id:ident), ($($path:path => $version:expr),*,)) => {
        lazy_static! {
            $(#[$attr])* $($vis)* static ref $id: std::collections::HashMap<&'static str, usize>
                = version_map_new!{ $($path => $version),*, };
        }
    }
}

/// Instantiate a `VersionGroupResolver` with specified version groups
#[macro_export]
macro_rules! version_group_resolver_new {
    ($(($api_group:expr, $api_version:expr) => { $($path:path => $version:expr),*, }),*,) => {
        {
            vec![
            $((
                (stringify!($api_group), stringify!($api_version)),
                Box::new(version_map_new!{ $($path => $version),*, })
            )),*,]
                .into_iter()
                .collect::<DefaultVersionGroupResolver<'static>>()
        }
    };
}

/// Instantiate a `VersionMap` with specified type's version
#[macro_export]
macro_rules! version_map_new {
    ($($path:path => $version:expr),*,) => {
        {
            vec![
                $(($crate::serde_version::exports::get_type_name::<$path>(), $version)),*,
            ]
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>()
        }
    };
}

/// Create an enum that maps an entry to a `VersionGroupURI<T>`
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

macro_rules! __version_group_enum {
    (($(#[$attr:meta])*), ($($vis:tt)*), ($id:ident), ($($entry:ident as $alias:expr => ($api_group:expr, $api_version:expr)),*,)) => {
        $(#[$attr])* $($vis)* enum $id {
            $(#[serde(alias = $alias)] $entry,),*
        }

        impl ::std::convert::From<$id> for $crate::VersionGroupURI<&'static str> {
            fn from(v: $id) -> Self {
                use ::std::convert::TryInto;
                match $v {$(
                    $entry => concat!(concat!(stringify!($api_group, ":"), stringify!($api_version))).try_into().unwrap()
                )*,}
            }
        }
    };
}

/// Define version groups
///
/// This create and enum and map its keys to a `VersionMap`
#[macro_export]
macro_rules! version_groups {
        ($($api:ident {
            $($id:ident as $alias:expr => {
                $($path:path => $version:expr),*,
            }),*,
        }),*,) => {
            $(
            #[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug)]
            pub enum $api {
                $(
                #[serde(alias = $alias)]
                $id
                ),*
            }

            impl $crate::version_util::VersionMapKey for $api {
                type VM = std::collections::HashMap<&'static str, usize>;

                fn to_version_map(&self) -> &Self::VM {
                    match *self {
                        $(
                            ApiVersion::$id => &$id,
                        )*
                    }
                }
            }

            $(
            lazy_static! {
                static ref $id: std::collections::HashMap<&'static str, usize> = {
                    vec![
                        $(($crate::serde_version::exports::get_type_name::<$path>(), $version)),*,
                    ]
                    .into_iter()
                    .collect::<std::collections::HashMap<_, _>>()
                };
            }
            )*
            )*
        };
    }
