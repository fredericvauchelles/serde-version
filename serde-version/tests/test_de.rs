#![feature(specialization)]

#[macro_use]
extern crate serde_version_derive;

use serde::Deserialize;
use serde_version::{
    DeserializeVersioned, Error, InvalidVersionError, VersionMap, VersionedDeserializer,
};

#[derive(Deserialize)]
#[serde(rename(deserialize = "A"))]
struct Av1 {
    a: u8,
}

#[derive(Deserialize)]
#[serde(default, rename(deserialize = "A"))]
struct Av2 {
    b: u8,
}

impl Default for Av2 {
    fn default() -> Self {
        Self { b: 5 }
    }
}

#[derive(Deserialize, PartialEq, DeserializeVersioned, Debug)]
#[serde(rename(deserialize = "A"))]
#[versions("Av1", version(type = "Av2", default))]
struct A {
    c: u8,
}

impl From<Av1> for A {
    fn from(v: Av1) -> Self {
        Self { c: v.a }
    }
}
impl From<Av2> for A {
    fn from(v: Av2) -> Self {
        Self { c: v.b }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
struct ContainsA {
    a: A,
}

fn execute_test<T: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
    value: T,
    from: &str,
    version_map: &VersionMap,
) {
    let mut ron_deserializer = ron::de::Deserializer::from_str(from).unwrap();
    let deserializer = VersionedDeserializer::new(&mut ron_deserializer, version_map);
    let de = <T as DeserializeVersioned>::deserialize_versioned(deserializer, version_map).unwrap();
    assert_eq!(value, de);
}

macro_rules! declare_tests_versions {
    (
        fail $name:ident ($($version_ty:expr => $version_num:expr),*) { $($ser:expr => $ty:ty: $value:expr,)+ }
        $($tt:tt)*
    ) => {
            #[test]
            fn $name() {
                let version_map = vec![$(($version_ty.to_owned(), $version_num),)*]
                    .into_iter().collect::<VersionMap>();
                $(
                    let mut ron_deserializer = ron::de::Deserializer::from_str($ser).unwrap();
                    let deserializer = VersionedDeserializer::new(&mut ron_deserializer, &version_map);
                    let de = <$ty as DeserializeVersioned>::deserialize_versioned(deserializer, &version_map).unwrap_err();
                    assert_eq!($value, de);
                )+
            }

            declare_tests_versions! { $($tt)* }
    };
    (
        $name:ident ($($version_ty:expr => $version_num:expr),*) { $($ser:expr => $value:expr,)+ }
        $($tt:tt)*
    ) => {
            #[test]
            fn $name() {
                let version_map = vec![$(($version_ty.to_owned(), $version_num),)*]
                    .into_iter().collect::<VersionMap>();
                $(
                    execute_test($value, $ser, &version_map);
                )+
            }

            declare_tests_versions! { $($tt)* }
    };
    () => { }
}

declare_tests_versions! {
    test_version ("A" => 1) {
        "A(a: 8)" => A { c: 8 },
        "ContainsA(a: A(a: 4))" => ContainsA { a: A { c: 4 }},
    }
    test_current_version ("A" => 3) {
        "A(c: 8)" => A { c: 8 },
        "ContainsA(a: A(c: 4))" => ContainsA { a: A { c: 4 }},
    }
    test_no_version () {
        "A(c: 8)" => A { c: 8 },
        "ContainsA(a: A(c: 4))" => ContainsA { a: A { c: 4 }},
    }
    test_default_version ("A" => 2) {
        "A(b: 8)" => A { c: 8 },
        "ContainsA(a: A(b: 4))" => ContainsA { a: A { c: 4 }},
        "A()" => A { c: 5 },
        "ContainsA(a: A())" => ContainsA { a: A { c: 5 }},
    }
    fail test_unknown_version ("A" => 4) {
        "A(b: 8)" => A: Error::InvalidVersionError(InvalidVersionError { version: 4, type_id: "A".to_owned() }),
        "ContainsA(a: A(b: 4))" => ContainsA: Error::DeserializeError(Error::DeserializeError(<ron::de::Error as serde::de::Error>::custom("Invalid version 4 for A"))),
    }
}
