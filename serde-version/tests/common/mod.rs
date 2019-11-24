#[macro_export]
macro_rules! declare_tests_versions {
    (
        fail $name:ident ($($vm:tt)*) { $($ty:ty: $value:expr =>$tokens:expr,)+ }
        $($tt:tt)*
    ) => {
            #[test]
            fn $name() {
                let version_map = build_version_map!($($vm)*);
                let version_map = get_version_map!(version_map, $($vm)*);

                $(
                    let mut de = serde_test::Deserializer::new($tokens);
                    let de_versioned = VersionedDeserializer::new(&mut de, version_map);
                    match <$ty as DeserializeVersioned>::deserialize_versioned(de_versioned, version_map) {
                        Ok(_) => {
                            panic!("tokens should have failed to deserialize")
                        }
                        Err(e) => assert_eq!(format!("{}", $value), format!("{}", e)),
                    };
                )+
            }

            declare_tests_versions! { $($tt)* }
    };
    (
        $name:ident ($($vm:tt)*) { $($ty:ty: $value:expr => $tokens:expr,)+ }
        $($tt:tt)*
    ) => {
            #[test]
            fn $name() {
                #[allow(unused_variables)]
                let version_map = build_version_map!($($vm)*);
                let version_map = get_version_map!(version_map, $($vm)*);

                $(
                    // Test ser/de roundtripping
                    let mut de = serde_test::Deserializer::new($tokens);
                    let de_versioned = ::serde_version::VersionedDeserializer::new(&mut de, version_map);
                    match <$ty as ::serde_version::DeserializeVersioned>::deserialize_versioned(de_versioned, version_map) {
                        Ok(v) => {
                            assert_eq!($value, v);
                            v
                        }
                        Err(e) => panic!("tokens failed to deserialize: {}", e),
                    };
                    if de.remaining() > 0 {
                        panic!("{} remaining tokens", de.remaining());
                    }
                )+
            }

            declare_tests_versions! { $($tt)* }
    };
    () => { }
}

macro_rules! build_version_map {
    // Build version map
    ($($version_ty:expr => $version_num:expr),*) => {
         vec![$(($version_ty, $version_num),)*]
                    .into_iter().collect::<DefaultVersionMap>()
    };
    // From static
    ($name:expr) => { () };
}

macro_rules! get_version_map {
    // Build version map
    ($id:ident, $($version_ty:expr => $version_num:expr),*) => {
        &$id
    };
    // From static
    ($id:ident, $name:expr) => {
        $name
    };
}
