#![feature(specialization)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_version_derive;

pub mod common;

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;

lazy_static! {
    static ref MY_VERSION_GROUP_V1: HashMap<&'static str, usize> = {
        vec![("A", 1), ("B", 2)]
            .into_iter()
            .collect::<HashMap<_, _>>()
    };
    static ref MY_VERSION_GROUP_V2: HashMap<&'static str, usize> = {
        vec![("A", 3), ("B", 2)]
            .into_iter()
            .collect::<HashMap<_, _>>()
    };
}

#[derive(Deserialize)]
#[serde(rename(deserialize = "A"))]
struct Av1 {
    a: u8,
}

#[derive(Deserialize)]
#[serde(rename(deserialize = "A"))]
struct Av2 {
    b: u8,
}

#[derive(Deserialize, PartialEq, DeserializeVersioned, Debug)]
#[serde(rename(deserialize = "A"))]
#[versions(
    v(index = 1, type = "Av1"),
    v(index = 3, type = "Av2"),
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
    aa: u8,
}

#[derive(Deserialize)]
#[serde(rename(deserialize = "B"))]
struct Bv2 {
    bb: u8,
}

#[derive(Deserialize, PartialEq, DeserializeVersioned, Debug)]
#[serde(rename(deserialize = "B"))]
#[versions(
    v(index = 1, type = "Bv1"),
    v(index = 2, type = "Bv2"),
    v(index = 3, self)
)]
struct B {
    cc: u8,
}

impl From<Bv1> for B {
    fn from(v: Bv1) -> Self {
        Self { cc: v.aa }
    }
}
impl From<Bv2> for B {
    fn from(v: Bv2) -> Self {
        Self { cc: v.bb }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
struct Container {
    a: A,
    b: B,
}

fn main() {
    use common::deserialize_test;

    deserialize_test(
        "Container(a: A(a: 8), b: B(bb: 3))",
        Container {
            a: A { c: 8 },
            b: B { cc: 3 },
        },
        &*MY_VERSION_GROUP_V1,
    );

    deserialize_test(
        "Container(a: A(b: 8), b: B(bb: 3))",
        Container {
            a: A { c: 8 },
            b: B { cc: 3 },
        },
        &*MY_VERSION_GROUP_V2,
    );
}
