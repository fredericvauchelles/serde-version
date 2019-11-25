use super::visitor::VersionedVisitor;
use super::Error;
use crate::version_map::{VersionMap, VersionMapIter};
use failure::_core::borrow::Borrow;
use serde::Deserializer;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

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
    VM: VersionMap,
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

macro_rules! impl_hashmap {
    (($($ly:tt),*), $($name:ty)*) => {
        impl<$($ly,)* T: Borrow<str> + Hash + Eq + Sync + 'static, S: BuildHasher + Sync> VersionMap
            for $($name)*
        {
            fn get(&self, type_id: &str) -> Option<usize> {
                std::collections::HashMap::get(self, type_id).cloned()
            }
        }
        impl<$($ly,)* 'i, T: Borrow<str> + Hash + Eq + 'i, S: BuildHasher + Sync> VersionMapIter<'i>
            for $($name)*
        {
            type Iter = std::iter::Map<
                std::collections::hash_map::Iter<'i, T, usize>,
                fn((&'i T, &'i usize)) -> (&'i str, usize),
            >;

            fn iter(&'i self) -> Self::Iter {
                HashMap::<T, usize, S>::iter(self).map(|(k, v)| (k.borrow(), *v))
            }
        }
    };
    () => {
        impl_hashmap!((), HashMap<T, usize, S>);
        impl_hashmap!(('a), &'a HashMap<T, usize, S>);
        impl_hashmap!(('a), &'a mut HashMap<T, usize, S>);
    };
}
impl_hashmap!();
