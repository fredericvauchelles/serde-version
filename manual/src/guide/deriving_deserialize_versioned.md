# Deriving `DeserializeVersioned`

You can generate the implementation of `DeserializeVersioned` with the derive macro.

```rust
// Only Deserialize is required for previous version
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

// This type has 3 version:
// - 1 = Av1
// - 3 = Av2
// - 4 = current
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
```