#![feature(min_specialization)]

#[macro_use]
extern crate serde_version_derive;
#[macro_use]
extern crate serde_version;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod common;

use serde::Deserialize;
use serde_test::Token;
use serde_version::VersionGroupResolver;
use std::fmt::Debug;

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
#[versions(
    v(index = 1, type = "Av1"),
    version(index = 3, type = "Av2", default),
    v(index = 4, self)
)]
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

#[derive(Deserialize)]
#[serde(rename(deserialize = "B"))]
struct Bv1 {
    a: u8,
}

#[derive(Deserialize)]
#[serde(default, rename(deserialize = "B"))]
struct Bv2 {
    b: u8,
}

impl Default for Bv2 {
    fn default() -> Self {
        Self { b: 5 }
    }
}

#[derive(Deserialize, PartialEq, DeserializeVersioned, Debug)]
#[serde(rename(deserialize = "B"))]
#[versions(
    v(index = 1, type = "Bv1"),
    version(index = 2, type = "Bv2", default),
    v(index = 4, self)
)]
struct B {
    c: u8,
}

impl From<Bv1> for B {
    fn from(v: Bv1) -> Self {
        Self { c: v.a }
    }
}
impl From<Bv2> for B {
    fn from(v: Bv2) -> Self {
        Self { c: v.b }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
struct ContainsBoth {
    a: A,
    b: B,
}

version_group_resolver_static! {
    pub VERSIONS = {
        ("serde_version.test" , "1.0.0") => { A => 1, B => 1, },
        ("serde_version.test" , "1.1.0") => { A => 3, B => 1, },
        ("serde_version.test" , "1.2.0") => { A => 4, B => 2, },
    }
}

version_group_enum! {
    #[derive(Deserialize)]
    enum Versions {
        V1 as "v1" => "serde_version.test:1.0.0",
        V2 as "v2" => "serde_version.test:1.1.0",
        V3 as "v3" => "serde_version.test:1.2.0",
    }
}

declare_tests_versions! {
    test_version_1 (VERSIONS.resolve(Versions::V1.into()).unwrap()) {
        A: A { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::I32(8),
            Token::MapEnd,
        ],
        B: B { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsBoth: ContainsBoth { a: A { c: 4 }, b: B { c: 6 } } => &[
            Token::Map { len: Some(2) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("a"),
                    Token::I32(4),
                Token::MapEnd,

                Token::Str("b"),
                Token::Map { len: Some(1) },
                    Token::Str("a"),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    test_version_2 (VERSIONS.resolve(Versions::V2.into()).unwrap()) {
        A: A { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("b"),
                Token::I32(8),
            Token::MapEnd,
        ],
        B: B { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsBoth: ContainsBoth { a: A { c: 4 }, b: B { c: 6 } } => &[
            Token::Map { len: Some(2) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("b"),
                    Token::I32(4),
                Token::MapEnd,

                Token::Str("b"),
                Token::Map { len: Some(1) },
                    Token::Str("a"),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    test_version_3 (VERSIONS.resolve(Versions::V3.into()).unwrap()) {
        A: A { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("c"),
                Token::I32(8),
            Token::MapEnd,
        ],
        B: B { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("b"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsBoth: ContainsBoth { a: A { c: 4 }, b: B { c: 6 } } => &[
            Token::Map { len: Some(2) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("c"),
                    Token::I32(4),
                Token::MapEnd,

                Token::Str("b"),
                Token::Map { len: Some(1) },
                    Token::Str("b"),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
}
