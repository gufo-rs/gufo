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

        impl std::convert::Into<$type> for $enum_name {
            fn into(self) -> $type {
                match self {
                    $(Self::$variant_name => $variant_value,)*
                    Self::Unknown(other) => other,
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
            pub struct [<Unknown $enum_name ValueError>]($type);

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

#[derive(Debug, PartialEq, Eq)]
pub struct AdditionOverflowError;
#[derive(Debug, PartialEq, Eq)]
pub struct SubstractionOverflowError;

pub trait U32Ext {
    fn usize(self) -> usize;
    fn i64(self) -> i64;
    fn safe_add(self, rhs: u32) -> Result<u32, AdditionOverflowError>;
    fn safe_sub(self, rhs: u32) -> Result<u32, SubstractionOverflowError>;
}

impl U32Ext for u32 {
    fn usize(self) -> usize {
        // Assume that systems are at least 32bit
        self.try_into().unwrap()
    }

    fn i64(self) -> i64 {
        self.into()
    }

    fn safe_add(self, rhs: u32) -> Result<u32, AdditionOverflowError> {
        self.checked_add(rhs).ok_or(AdditionOverflowError)
    }

    fn safe_sub(self, rhs: u32) -> Result<u32, SubstractionOverflowError> {
        self.checked_sub(rhs).ok_or(SubstractionOverflowError)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConversionOverflowError;

pub trait I64Ext {
    fn u32(self) -> Result<u32, ConversionOverflowError>;
    fn safe_add(self, rhs: i64) -> Result<i64, AdditionOverflowError>;
}

impl I64Ext for i64 {
    fn u32(self) -> Result<u32, ConversionOverflowError> {
        self.try_into().map_err(|_| ConversionOverflowError)
    }

    fn safe_add(self, rhs: i64) -> Result<i64, AdditionOverflowError> {
        self.checked_add(rhs).ok_or(AdditionOverflowError)
    }
}
