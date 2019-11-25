#![feature(specialization)]

extern crate lazy_static;
#[cfg_attr(feature = "toml-support", macro_use)]
extern crate serde;
#[cfg_attr(feature = "toml-support", macro_use)]
extern crate serde_version;
#[cfg_attr(feature = "toml-support", macro_use)]
extern crate serde_version_derive;

#[cfg(feature = "toml-support")]
pub mod toml;
