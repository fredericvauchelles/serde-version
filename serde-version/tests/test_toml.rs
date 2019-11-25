#![feature(specialization)]

#[macro_use]
extern crate serde_version_derive;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_version;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "toml-support")]
pub mod toml;
