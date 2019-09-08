use super::Error;
use super::{VersionMap, VersionedDeserializer};
use crate::seed::VersionedSeed;
use crate::DeserializeVersioned;
use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::{Deserialize, Deserializer};

/// Wrap a visitor to wrap seed or call specialized methods
pub struct VersionedVisitor<'v, V> {
    visitor: V,
    version_map: &'v VersionMap,
}

impl<'v, V> VersionedVisitor<'v, V> {
    pub fn new(visitor: V, version_map: &'v VersionMap) -> Self {
        Self {
            visitor,
            version_map,
        }
    }
}

macro_rules! forward_visit {
        ($name:ident, $ty:ty) => {
            #[inline]
            fn $name<E>(self, v: $ty) -> Result<V::Value, E>
                where E: serde::de::Error
            {
                self.visitor.$name(v)
            }
        }
    }

impl<'de, V> Visitor<'de> for VersionedVisitor<'de, V>
where
    V: Visitor<'de>,
{
    type Value = V::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.visitor.expecting(formatter)
    }

    forward_visit!(visit_bool, bool);
    forward_visit!(visit_i8, i8);
    forward_visit!(visit_i16, i16);
    forward_visit!(visit_i32, i32);
    forward_visit!(visit_i64, i64);
    forward_visit!(visit_u8, u8);
    forward_visit!(visit_u16, u16);
    forward_visit!(visit_u32, u32);
    forward_visit!(visit_u64, u64);
    forward_visit!(visit_f32, f32);
    forward_visit!(visit_f64, f64);
    forward_visit!(visit_char, char);
    forward_visit!(visit_bytes, &[u8]);
    forward_visit!(visit_byte_buf, Vec<u8>);
    forward_visit!(visit_str, &str);
    forward_visit!(visit_string, String);
    forward_visit!(visit_borrowed_str, &'de str);

    #[inline]
    fn visit_unit<E>(self) -> Result<V::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_unit()
    }

    #[inline]
    fn visit_none<E>(self) -> Result<V::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_none()
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.visitor
            .visit_some(VersionedDeserializer::new(deserializer, self.version_map))
            .map_err(|err| err.into_error())
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.visitor
            .visit_newtype_struct(VersionedDeserializer::new(deserializer, self.version_map))
            .map_err(|err| err.into_error())
    }

    #[inline]
    fn visit_seq<V2>(self, visitor: V2) -> Result<V::Value, V2::Error>
    where
        V2: SeqAccess<'de>,
    {
        let visitor = VersionedVisitor {
            visitor,
            version_map: self.version_map,
        };
        self.visitor
            .visit_seq(visitor)
            .map_err(|err| err.into_error())
    }

    #[inline]
    fn visit_map<V2>(self, visitor: V2) -> Result<V::Value, V2::Error>
    where
        V2: MapAccess<'de>,
    {
        let visitor = VersionedVisitor {
            visitor,
            version_map: self.version_map,
        };
        self.visitor
            .visit_map(visitor)
            .map_err(|err| err.into_error())
    }

    #[inline]
    fn visit_enum<V2>(self, visitor: V2) -> Result<V::Value, V2::Error>
    where
        V2: EnumAccess<'de>,
    {
        let visitor = VersionedVisitor {
            visitor,
            version_map: self.version_map,
        };
        self.visitor
            .visit_enum(visitor)
            .map_err(|err| err.into_error())
    }
}

impl<'de, V> SeqAccess<'de> for VersionedVisitor<'de, V>
where
    V: SeqAccess<'de>,
{
    type Error = Error<V::Error>;

    #[inline]
    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<<T as DeserializeSeed<'de>>::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let seed = VersionedSeed::new(seed, self.version_map);
        self.visitor
            .next_element_seed(seed)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        <T as DeserializeVersioned<'de>>::next_element(self, self.version_map)
            .map_err(|err| err.reduce())
    }
}

impl<'de, V> MapAccess<'de> for VersionedVisitor<'de, V>
where
    V: MapAccess<'de>,
{
    type Error = Error<V::Error>;

    #[inline]
    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> Result<Option<<K as DeserializeSeed<'de>>::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let seed = VersionedSeed::new(seed, self.version_map);
        self.visitor
            .next_key_seed(seed)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    fn next_value_seed<S>(
        &mut self,
        seed: S,
    ) -> Result<<S as DeserializeSeed<'de>>::Value, Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        let seed = VersionedSeed::new(seed, self.version_map);
        self.visitor
            .next_value_seed(seed)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    #[allow(clippy::type_complexity)]
    fn next_entry_seed<K, V2>(
        &mut self,
        kseed: K,
        vseed: V2,
    ) -> Result<Option<(K::Value, V2::Value)>, Self::Error>
    where
        K: DeserializeSeed<'de>,
        V2: DeserializeSeed<'de>,
    {
        let kseed = VersionedSeed::new(kseed, self.version_map);
        let vseed = VersionedSeed::new(vseed, self.version_map);
        self.visitor
            .next_entry_seed(kseed, vseed)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    fn next_key<K>(&mut self) -> Result<Option<K>, Self::Error>
    where
        K: Deserialize<'de>,
    {
        <K as DeserializeVersioned<'de>>::next_key(self, self.version_map)
            .map_err(|err| err.reduce())
    }

    #[inline]
    fn next_value<V2>(&mut self) -> Result<V2, Self::Error>
    where
        V2: Deserialize<'de>,
    {
        <V2 as DeserializeVersioned<'de>>::next_value(self, self.version_map)
            .map_err(|err| err.reduce())
    }

    fn size_hint(&self) -> Option<usize> {
        self.visitor.size_hint()
    }
}

impl<'de, V> EnumAccess<'de> for VersionedVisitor<'de, V>
where
    V: EnumAccess<'de>,
{
    type Error = Error<V::Error>;
    type Variant = VersionedVisitor<'de, V::Variant>;

    #[inline]
    #[allow(clippy::type_complexity)]
    fn variant_seed<S>(
        self,
        seed: S,
    ) -> Result<(S::Value, VersionedVisitor<'de, V::Variant>), Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        let seed = VersionedSeed::new(seed, self.version_map);
        match self.visitor.variant_seed(seed) {
            Ok((value, variant)) => {
                let variant = VersionedVisitor {
                    visitor: variant,
                    version_map: self.version_map,
                };
                Ok((value, variant))
            }
            Err(e) => Err(Error::DeserializeError(e)),
        }
    }

    #[inline]
    fn variant<V2>(self) -> Result<(V2, Self::Variant), Self::Error>
    where
        V2: Deserialize<'de>,
    {
        let version_map = self.version_map;
        <V2 as DeserializeVersioned<'de>>::variant(self, version_map).map_err(|err| err.reduce())
    }
}

impl<'de, V> VariantAccess<'de> for VersionedVisitor<'de, V>
where
    V: VariantAccess<'de>,
{
    type Error = Error<V::Error>;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        self.visitor.unit_variant().map_err(Error::DeserializeError)
    }

    #[inline]
    fn newtype_variant_seed<S>(self, seed: S) -> Result<S::Value, Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        let seed = VersionedSeed::new(seed, self.version_map);
        self.visitor
            .newtype_variant_seed(seed)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    fn tuple_variant<V2>(self, len: usize, visitor: V2) -> Result<V2::Value, Self::Error>
    where
        V2: Visitor<'de>,
    {
        let visitor = VersionedVisitor {
            visitor,
            version_map: self.version_map,
        };
        self.visitor
            .tuple_variant(len, visitor)
            .map_err(Error::DeserializeError)
    }

    #[inline]
    fn struct_variant<V2>(
        self,
        fields: &'static [&'static str],
        visitor: V2,
    ) -> Result<V2::Value, Self::Error>
    where
        V2: Visitor<'de>,
    {
        let visitor = VersionedVisitor {
            visitor,
            version_map: self.version_map,
        };
        self.visitor
            .struct_variant(fields, visitor)
            .map_err(Error::DeserializeError)
    }
}