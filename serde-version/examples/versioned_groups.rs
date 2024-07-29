#![feature(min_specialization)]

#[macro_use]
extern crate serde_version_derive;
#[macro_use]
extern crate serde_version;
#[macro_use]
extern crate lazy_static;

pub mod common;

use serde::Deserialize;
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

// Define a version group resolver as a static variable
version_group_resolver_static! {
    pub VERSIONS = {
        ("version_group.example" , "1.0.0") => { A => 1, B => 1, },
        ("version_group.example" , "1.1.0") => { A => 3, B => 1, },
        ("version_group.example" , "1.2.0") => { A => 4, B => 2, },
    }
}

// Define an enum to have an easy way to get the version uris
version_group_enum! {
    #[derive(Deserialize)]
    enum Versions {
        V1 as "v1" => "version_group.example:1.0.0",
        V2 as "v2" => "version_group.example:1.1.0",
        V3 as "v3" => "version_group.example:1.2.0",
    }
}

fn main() {
    use common::deserialize_test;

    // V1
    deserialize_test(
        "A(a: 8)",
        A { c: 8 },
        VERSIONS.resolve(Versions::V1.into()).unwrap(),
    );
    deserialize_test(
        "B(a: 8)",
        B { c: 8 },
        VERSIONS.resolve(Versions::V1.into()).unwrap(),
    );
    deserialize_test(
        "ContainsBoth(a: A(a: 9), b: B(a: 10))",
        ContainsBoth {
            a: A { c: 9 },
            b: B { c: 10 },
        },
        VERSIONS.resolve(Versions::V1.into()).unwrap(),
    );

    // V2
    deserialize_test(
        "A(b: 8)",
        A { c: 8 },
        VERSIONS.resolve(Versions::V2.into()).unwrap(),
    );
    deserialize_test(
        "B(a: 8)",
        B { c: 8 },
        VERSIONS.resolve(Versions::V2.into()).unwrap(),
    );
    deserialize_test(
        "ContainsBoth(a: A(b: 9), b: B(a: 10))",
        ContainsBoth {
            a: A { c: 9 },
            b: B { c: 10 },
        },
        VERSIONS.resolve(Versions::V2.into()).unwrap(),
    );
}
