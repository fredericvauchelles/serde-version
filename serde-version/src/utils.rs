#[macro_export]
macro_rules! impl_from_enum {
    ($($enum:ident::$variant:ident => $from:ty),*,) => {
        $(
        impl ::std::convert::From<$from> for $enum {
            fn from(v: $from) -> Self {
                $enum::$variant(v)
            }
        }
        )*
    }
}
