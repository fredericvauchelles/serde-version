#![feature(min_specialization)]

#[macro_use]
extern crate serde_version_derive;

#[macro_use]
mod common;

use serde::Deserialize;
use serde_test::Token;
use serde_version::{
    DefaultVersionMap, DeserializeVersioned, InvalidVersionError, VersionedDeserializer,
};
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

#[derive(Deserialize, PartialEq, Debug)]
struct ContainsA {
    a: A,
}
declare_tests_versions! {
    test_version ("test_de::A" => 1) {
        A: A { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsA: ContainsA { a: A { c: 4 }} => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("a"),
                    Token::I32(4),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    test_current_version ("test_de::A" => 4) {
        A: A { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("c"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsA: ContainsA { a: A { c: 4 }} => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("c"),
                    Token::I32(4),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    test_no_version () {
        A: A { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("c"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsA: ContainsA { a: A { c: 4 }} => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("c"),
                    Token::I32(4),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    test_default_version ("test_de::A" => 3) {
        A: A { c: 8 }  => &[
            Token::Map { len: Some(1) },
                Token::Str("b"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsA: ContainsA { a: A { c: 4 }} => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("b"),
                    Token::I32(4),
                Token::MapEnd,
            Token::MapEnd,
        ],
        A: A { c: 5 }  => &[
            Token::Map { len: Some(0) },
            Token::MapEnd,
        ],
        ContainsA: ContainsA { a: A { c: 5 }} => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
    fail test_unknown_version ("test_de::A" => 5) {
        A: InvalidVersionError { version: 5, type_id: "test_de::A".to_owned() } => &[
            Token::Map { len: Some(1) },
                Token::Str("b"),
                Token::I32(8),
            Token::MapEnd,
        ],
        ContainsA: InvalidVersionError { version: 5, type_id: "test_de::A".to_owned() } => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::Map { len: Some(1) },
                    Token::Str("b"),
                    Token::I32(4),
                Token::MapEnd,
            Token::MapEnd,
        ],
    }
}
