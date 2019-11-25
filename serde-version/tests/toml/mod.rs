#[derive(Deserialize)]
#[serde(rename = "A")]
struct Av1 {
    a: usize,
}

#[derive(Serialize, Deserialize, DeserializeVersioned, PartialEq, Debug)]
#[versions(v(index = 1, type = "Av1"), v(index = 2, self))]
struct A {
    b: usize,
}

impl From<Av1> for A {
    fn from(v: Av1) -> Self {
        Self { b: v.a }
    }
}

#[derive(Deserialize)]
#[serde(rename = "B")]
struct Bv1 {
    a: usize,
}

#[derive(Serialize, Deserialize, DeserializeVersioned, PartialEq, Debug)]
#[versions(v(index = 1, type = "Bv1"), v(index = 2, self))]
struct B {
    b: usize,
}

impl From<Bv1> for B {
    fn from(v: Bv1) -> Self {
        Self { b: v.a }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Container {
    a: A,
    b: B,
}

version_group_resolver_static! {
    pub VERSIONS = {
        ("a" , "1") => { A => 1, },
        ("a" , "2") => { A => 2, },
        ("b" , "1") => { B => 1, },
        ("b" , "2") => { B => 2, },
    }
}

version_group_enum! {
    #[derive(Deserialize)]
    enum Versions {
        A1 as "av1" => "a:1",
        A2 as "av2" => "a:2",
        B1 as "bv1" => "b:1",
        B2 as "bv2" => "b:2",
    }
}

macro_rules! declare_tests {
    ($name:ident { $($value:expr => $toml:expr)* }) => {
        #[test]
        fn $name() {
            $({
                let input = $toml;
                let de: Container = serde_version::toml::deserialize(input, &*VERSIONS).unwrap();
                assert_eq!($value, de);
            })*
        }
    };
}

declare_tests! {
    deserialize_works {
        Container { a: A { b: 5 }, b: B { b: 3 } } => r#"v = ["a:1", "b:2"]
[a]
a = 5

[b]
b = 3
"#
        Container { a: A { b: 5 }, b: B { b: 3 } } => r#"v = ["a:2", "b:2"]
[a]
b = 5

[b]
b = 3
"#
    }
}
