use super::visitor::VersionedVisitor;
use super::Error;
use failure::_core::borrow::Borrow;
use serde::Deserializer;
use std::collections::HashMap;
use std::hash::Hash;

/// Maps the version number for each deserialization type name
pub trait VersionMap {
    fn get(&self, type_id: &str) -> Option<usize>;
}
pub type DefaultVersionMap = HashMap<String, usize>;

/// A wrapper around a deserialize to support the deserialization.
///
/// This deserializer will wrap all calls where specialization is required. (Like
/// `next_element`, `next_value`, ...)
pub struct VersionedDeserializer<'de, D, VM>
where
    D: Deserializer<'de>,
{
    deserializer: D,
    version_map: &'de VM,
    marker: std::marker::PhantomData<&'de usize>,
}

impl<'de, D, VM> VersionedDeserializer<'de, D, VM>
where
    D: Deserializer<'de>,
{
    pub fn new(deserializer: D, version_map: &'de VM) -> Self {
        Self {
            deserializer,
            version_map,
            marker: std::marker::PhantomData,
        }
    }
}

macro_rules! forward_deserialize {
    ($name:ident) => {forward_deserialize!($name, );};
    ($name:ident, $($arg:tt => $ty:ty),*) => {
        fn $name<V>(self, $($arg: $ty,)* visitor: V) -> Result<V::Value, Error<D::Error>>
            where V: serde::de::Visitor<'de>
        {
            let visitor = VersionedVisitor::new(
                visitor,
                self.version_map,
            );
            self.deserializer.$name($($arg,)* visitor).map_err(Error::DeserializeError)
        }
    }
}

impl<'de, D: Deserializer<'de>, VM: VersionMap> Deserializer<'de>
    for VersionedDeserializer<'de, D, VM>
{
    type Error = Error<D::Error>;

    forward_deserialize!(deserialize_any);
    forward_deserialize!(deserialize_bool);
    forward_deserialize!(deserialize_u8);
    forward_deserialize!(deserialize_u16);
    forward_deserialize!(deserialize_u32);
    forward_deserialize!(deserialize_u64);
    forward_deserialize!(deserialize_i8);
    forward_deserialize!(deserialize_i16);
    forward_deserialize!(deserialize_i32);
    forward_deserialize!(deserialize_i64);
    forward_deserialize!(deserialize_f32);
    forward_deserialize!(deserialize_f64);
    forward_deserialize!(deserialize_char);
    forward_deserialize!(deserialize_str);
    forward_deserialize!(deserialize_string);
    forward_deserialize!(deserialize_unit);
    forward_deserialize!(deserialize_option);
    forward_deserialize!(deserialize_seq);
    forward_deserialize!(deserialize_bytes);
    forward_deserialize!(deserialize_byte_buf);
    forward_deserialize!(deserialize_map);
    forward_deserialize!(deserialize_unit_struct, name => &'static str);
    forward_deserialize!(deserialize_newtype_struct, name => &'static str);
    forward_deserialize!(deserialize_tuple_struct, name => &'static str, len => usize);
    forward_deserialize!(deserialize_struct,
                         name => &'static str,
                         fields => &'static [&'static str]);
    forward_deserialize!(deserialize_identifier);
    forward_deserialize!(deserialize_tuple, len => usize);
    forward_deserialize!(deserialize_enum,
                         name => &'static str,
                         variants => &'static [&'static str]);
    forward_deserialize!(deserialize_ignored_any);
}

impl<T: Borrow<str> + Hash + Eq> VersionMap for HashMap<T, usize> {
    fn get(&self, type_id: &str) -> Option<usize> {
        std::collections::HashMap::get(self, type_id).cloned()
    }
}

impl<'a, T: Borrow<str> + Hash + Eq> VersionMap for &'a HashMap<T, usize> {
    fn get(&self, type_id: &str) -> Option<usize> {
        std::collections::HashMap::get(self, type_id).cloned()
    }
}

impl<'a, T: Borrow<str> + Hash + Eq> VersionMap for &'a mut HashMap<T, usize> {
    fn get(&self, type_id: &str) -> Option<usize> {
        std::collections::HashMap::get(self, type_id).cloned()
    }
}
