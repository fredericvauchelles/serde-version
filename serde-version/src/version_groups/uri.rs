use failure::_core::fmt::{Error, Formatter};
use serde::{de, Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::Display;
use std::ops::Deref;

#[derive(Serialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct VersionGroupURI<'a> {
    #[serde(borrow)]
    source: Cow<'a, str>,
    #[serde(skip)]
    index: usize,
}
impl<'a> VersionGroupURI<'a> {
    pub fn api_group(&self) -> &str {
        &self.source[..self.index]
    }
    pub fn version(&self) -> &str {
        &self.source[(self.index + 1)..]
    }
}

impl<'a> Display for VersionGroupURI<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.source)
    }
}

impl<'a> VersionGroupURI<'a> {
    pub fn to_static(&self) -> VersionGroupURI<'static> {
        VersionGroupURI {
            source: Cow::Owned(self.source.clone().into_owned()),
            index: self.index,
        }
    }
}

impl<'a> ToString for VersionGroupURI<'a> {
    fn to_string(&self) -> String {
        self.source.clone().into_owned()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Fail)]
#[fail(display = "Invalid format {}, expected \"api_group:version\"", source)]
pub struct TryFromError {
    pub source: String,
}
impl<'a> TryFrom<Cow<'a, str>> for VersionGroupURI<'a> {
    type Error = TryFromError;

    fn try_from(source: Cow<'a, str>) -> Result<Self, Self::Error> {
        if let Some(index) = source.find(':') {
            if index == 0 || index + 1 >= source.len() {
                Err(TryFromError {
                    source: source.into_owned(),
                })
            } else {
                Ok(VersionGroupURI { source, index })
            }
        } else {
            Err(TryFromError {
                source: source.into_owned(),
            })
        }
    }
}
impl<'a> TryFrom<&'a str> for VersionGroupURI<'a> {
    type Error = TryFromError;

    fn try_from(source: &'a str) -> Result<Self, Self::Error> {
        VersionGroupURI::try_from(Cow::Borrowed(source))
    }
}
impl<'a> TryFrom<String> for VersionGroupURI<'a> {
    type Error = TryFromError;

    fn try_from(source: String) -> Result<Self, Self::Error> {
        VersionGroupURI::try_from(Cow::Owned(source))
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for VersionGroupURI<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as de::Deserializer<'de>>::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct VersionGroupURIVisitor<'b>(std::marker::PhantomData<&'b u8>);

        impl<'b, 'de: 'b> de::Visitor<'de> for VersionGroupURIVisitor<'b> {
            type Value = VersionGroupURI<'b>;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> Result<(), std::fmt::Error> {
                formatter.write_str("\"api_group:version\"")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                VersionGroupURI::try_from(Cow::Owned(v.to_string()))
                    .map(std::convert::Into::into)
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &self))
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                VersionGroupURI::try_from(Cow::Borrowed(v))
                    .map(std::convert::Into::into)
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &self))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                VersionGroupURI::try_from(Cow::Owned(v)).map_err(|err| {
                    de::Error::invalid_value(de::Unexpected::Str(&err.source), &self)
                })
            }
        }

        deserializer.deserialize_any(VersionGroupURIVisitor::<'a>(std::marker::PhantomData))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct VersionGroupURIs<'a> {
    #[serde(borrow)]
    v: Vec<VersionGroupURI<'a>>,
}
impl<'a> VersionGroupURIs<'a> {
    fn versions(&self) -> &[VersionGroupURI<'a>] {
        &self.v
    }
}
impl<'a> Deref for VersionGroupURIs<'a> {
    type Target = [VersionGroupURI<'a>];

    fn deref(&self) -> &Self::Target {
        self.versions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_de_tokens, Token};

    macro_rules! declare_tests {
        ($(
            $(#[$cfg:meta])*
            $name:ident { $($value:expr => $tokens:expr,)+ }
        )+) => {
            $(
                $(#[$cfg])*
                #[test]
                fn $name() {
                    $(
                        // Test ser/de roundtripping
                        assert_de_tokens(&$value, $tokens);
                    )+
                }
            )+
        }
    }

    declare_tests! {
        test_uri {
            VersionGroupURI { source: Cow::Borrowed("my.api_group:1.0.0"), index: 12 } => &[
                Token::Str("my.api_group:1.0.0"),
            ],
            VersionGroupURI { source: Cow::Borrowed("my.api_group:1.0.0"), index: 12 } => &[
                Token::BorrowedStr("my.api_group:1.0.0"),
            ],
            VersionGroupURI { source: Cow::Borrowed("my.api_group:1.0.0"), index: 12 } => &[
                Token::String("my.api_group:1.0.0"),
            ],
        }
        test_uris {
            VersionGroupURIs { v: vec![
                VersionGroupURI { source: Cow::Borrowed("my.api_group:1.0.0"), index: 12 },
                VersionGroupURI { source: Cow::Borrowed("my.second.api_group:1.2.0"), index: 19 },
            ] } => &[
                Token::Map { len: Some(1) },
                    Token::Str("v"),
                    Token::Seq { len: Some(2) },
                        Token::Str("my.api_group:1.0.0"),
                        Token::Str("my.second.api_group:1.2.0"),
                    Token::SeqEnd,
                Token::MapEnd,
            ],
        }
    }

    #[test]
    fn uri_from_str_works() {
        {
            let uri = VersionGroupURI::try_from("my.api_group:1.0.0").unwrap();
            assert_eq!("my.api_group", uri.api_group());
            assert_eq!("1.0.0", uri.version());
        }
    }

    #[test]
    fn uri_from_string_works() {
        {
            let uri = VersionGroupURI::try_from("my.api_group:1.0.0".to_string()).unwrap();
            assert_eq!("my.api_group", uri.api_group());
            assert_eq!("1.0.0", uri.version());
        }
    }

    #[test]
    fn uri_from_cow_works() {
        {
            let uri = VersionGroupURI::try_from(Cow::Borrowed("my.api_group:1.0.0")).unwrap();
            assert_eq!("my.api_group", uri.api_group());
            assert_eq!("1.0.0", uri.version());
        }
    }

    #[quickcheck]
    fn qc_uri_from_str_works(input: String) {
        qc_uri_from_str_works_fn(&input);
    }

    fn qc_uri_from_str_works_fn(input: &str) {
        let result = VersionGroupURI::try_from(input);

        if let Some(index) = input.find(':') {
            if index == 0 || index + 1 >= input.len() {
                assert!(result.is_err(), "Missing api_group or version.");
            } else {
                if let Ok(uri) = result {
                    assert_eq!(&input[..index], uri.api_group());
                    assert_eq!(&input[(index + 1)..], uri.version());
                } else {
                    unreachable!("Parsing should have succeeded.");
                }
            }
        } else {
            assert!(result.is_err(), "Missing ':' token.");
        }
    }
}
