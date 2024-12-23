/// Adds conversions `from` and `into` integer to enums
///
/// Takes an enum that must have a `#[repr()]` as first meta field and assigns a
/// value to all enum variants.
///
/// ```
/// # use gufo_common::utils::convertible_enum;
/// convertible_enum!(
///     #[repr(u8)]
///     #[derive(Debug, PartialEq)]
///     pub enum Test {
///         Val1 = 1,
///         Val2 = 2,
///     }
/// );
/// let int: u8 = Test::Val2.into();
/// assert_eq!(int, 2);
/// assert_eq!(Test::from(2), Test::Val2);
/// assert_eq!(Test::from(3), Test::Unknown(3));
/// ```
#[macro_export]
macro_rules! convertible_enum {
    (#[repr($type:ty)]$(#[$meta:meta])* $visibility:vis enum $enum_name:ident {
        $($(#[$variant_meta:meta])* $variant_name:ident = $variant_value:expr,)*
    }) => {
        #[repr($type)]
        $(#[$meta])*
        $visibility enum $enum_name {
            $($(#[$variant_meta])* $variant_name = $variant_value,)*
            Unknown($type)
        }

        impl std::convert::From<$type> for $enum_name {
            fn from(v: $type) -> Self {
                if false { unreachable!() }
                $( else if v == $variant_value { Self::$variant_name } )*
                else { Self::Unknown(v) }
            }
        }

        impl std::convert::From<$enum_name> for $type {
            fn from(value: $enum_name) -> $type {
                match value {
                    $($enum_name::$variant_name => $variant_value,)*
                    $enum_name::Unknown(other) => other,
                }
            }
        }
    }
}

/// Adds conversions `try_from` and `into` integer to enums
///
/// Takes an enum that must have a `#[repr()]` as first meta field and assigns a
/// value to all enum variants.
///
/// ```
/// # use gufo_common::utils::maybe_convertible_enum;
/// maybe_convertible_enum!(
///     #[repr(u8)]
///     #[derive(Debug, PartialEq)]
///     pub enum Test {
///         Val1 = 1,
///         Val2 = 2,
///     }
/// );
/// let int: u8 = Test::Val2.into();
/// assert_eq!(int, 2);
/// assert_eq!(Test::try_from(2), Ok(Test::Val2));
/// assert_eq!(Test::try_from(3), Err(UnknownTestValueError(3)));
/// ```
#[macro_export]
macro_rules! maybe_convertible_enum {
    (#[repr($type:ty)]$(#[$meta:meta])* $visibility:vis enum $enum_name:ident {
        $($(#[$variant_meta:meta])* $variant_name:ident = $variant_value:expr,)*
    }) => {
        #[repr($type)]
        $(#[$meta])*
        $visibility enum $enum_name {
            $($(#[$variant_meta])* $variant_name = $variant_value,)*
        }

        paste::paste! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct [<Unknown $enum_name ValueError>](pub $type);

            impl std::fmt::Display for [<Unknown $enum_name ValueError>] {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, concat!("Enum '", stringify!($enum_name), "' has no variant with value '{}'"), self.0)
                }
            }

            impl std::error::Error for [<Unknown $enum_name ValueError>] {}

            /// Create enum from it's discriminant value
            impl std::convert::TryFrom<$type> for $enum_name {
                type Error =  [<Unknown $enum_name ValueError>];
                fn try_from(v: $type) -> Result<Self, Self::Error> {
                    match v {
                        $($variant_value => Ok(Self::$variant_name),)*
                        other => Err([<Unknown $enum_name ValueError>](other)),
                    }
                }
            }
        }

        /// Convert enum to it's discriminant value
        impl From<$enum_name> for $type {
            fn from(v: $enum_name) -> $type {
                match v {
                    $($enum_name::$variant_name => $variant_value,)*
                }
            }
        }
    }
}

pub use {convertible_enum, maybe_convertible_enum};
