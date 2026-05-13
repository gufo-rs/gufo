use std::fmt::Display;

use zerocopy::{ByteOrder, FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, U16, U32, U64};

use crate::Error;

#[derive(Debug, Clone, Copy)]
pub enum Endieness {
    Big,
    Litte,
}

impl Endieness {
    pub fn u16_from_bytes(&self, bytes: &[u8]) -> Result<u16, Error> {
        let bytes = bytes
            .try_into()
            .map_err(|_| Error::InputDataWrongLength(bytes.len(), 2))?;

        match self {
            Self::Big => Ok(u16::from_be_bytes(bytes)),
            Self::Litte => Ok(u16::from_le_bytes(bytes)),
        }
    }

    pub fn u32_from_bytes(&self, bytes: &[u8]) -> Result<u32, Error> {
        let bytes = bytes
            .try_into()
            .map_err(|_| Error::InputDataWrongLength(bytes.len(), 4))?;

        match self {
            Self::Big => Ok(u32::from_be_bytes(bytes)),
            Self::Litte => Ok(u32::from_le_bytes(bytes)),
        }
    }

    pub fn i32_from_bytes(&self, bytes: &[u8]) -> Result<i32, Error> {
        let bytes = bytes
            .try_into()
            .map_err(|_| Error::InputDataWrongLength(bytes.len(), 4))?;

        match self {
            Self::Big => Ok(i32::from_be_bytes(bytes)),
            Self::Litte => Ok(i32::from_le_bytes(bytes)),
        }
    }
}

#[cfg(feature = "chrono")]
pub fn datetime(
    datetime: String,
    subsec: Option<String>,
    offset: Option<String>,
) -> Result<gufo_common::datetime::DateTime, Error> {
    let mut datetime = datetime.replacen(':', "-", 2).replacen(' ', "T", 1);

    if let Some(subsec) = subsec {
        // Remove NULL as well since iPhone 15 and HTC ONE have a leading NULL in this
        // field
        let subsec = subsec.trim();
        if !subsec.is_empty() {
            datetime.push('.');
            datetime.push_str(subsec);
        }
    }

    let use_offset;

    // Add offset (timezone)
    if let Some(offset) = offset {
        datetime.push_str(&offset);
        use_offset = true;
    } else {
        // Add an offset to allow parser to work
        datetime.push('Z');
        use_offset = false;
    }

    let x = chrono::DateTime::parse_from_rfc3339(&datetime)
        .map_err(|err| Error::Other(format!("Failed to parse datetime: {err}")))?;

    Ok(if use_offset {
        gufo_common::datetime::DateTime::FixedOffset(x)
    } else {
        gufo_common::datetime::DateTime::Naive(x.naive_utc())
    })
}

#[track_caller]
pub fn handle_error<T, E: Display>(x: Result<Option<T>, E>) -> Option<T> {
    match x {
        Ok(res) => res,
        Err(err) => {
            #[cfg(feature = "tracing")]
            tracing::debug!("Lookup error: {err}");
            None
        }
    }
}

#[track_caller]
pub fn handle_error_<T, E: Display>(x: Result<T, E>) -> Option<T> {
    match x {
        Ok(res) => Some(res),
        Err(err) => {
            #[cfg(feature = "tracing")]
            tracing::debug!("Lookup error: {err}");
            None
        }
    }
}

pub trait UsizeConversion: Sized {
    fn try_to_usize(&self) -> Result<usize, Error>;
    fn try_from_usize(u: usize) -> Result<Self, Error>;
}

impl<O: ByteOrder> UsizeConversion for U16<O> {
    fn try_to_usize(&self) -> Result<usize, Error> {
        Ok(self.get() as usize)
    }

    fn try_from_usize(u: usize) -> Result<Self, Error> {
        Ok(Self::new(u.try_into()?))
    }
}

impl<O: ByteOrder> UsizeConversion for U32<O> {
    fn try_to_usize(&self) -> Result<usize, Error> {
        self.get().try_into().map_err(Into::into)
    }

    fn try_from_usize(u: usize) -> Result<Self, Error> {
        Ok(Self::new(u.try_into()?))
    }
}

impl<O: ByteOrder> UsizeConversion for U64<O> {
    fn try_to_usize(&self) -> Result<usize, Error> {
        self.get().try_into().map_err(Into::into)
    }

    fn try_from_usize(u: usize) -> Result<Self, Error> {
        Ok(Self::new(u.try_into()?))
    }
}

pub trait IndexType:
    FromBytes + IntoBytes + Immutable + KnownLayout + Unaligned + UsizeConversion + 'static
{
    type NEntries: FromBytes
        + IntoBytes
        + Immutable
        + KnownLayout
        + Unaligned
        + UsizeConversion
        + 'static;
}

impl<O: ByteOrder + 'static> IndexType for U32<O> {
    type NEntries = U16<O>;
}
impl<O: ByteOrder + 'static> IndexType for U64<O> {
    type NEntries = U64<O>;
}

#[macro_export]
macro_rules! forall_formats {
    ($value:expr, $varname:ident, $function:expr) => {
        match $value {
            Self::Le32($varname) => $function,
            Self::Be32($varname) => $function,
            Self::Le64($varname) => $function,
            Self::Be64($varname) => $function,
        }
    };
}

#[macro_export]
macro_rules! forall_formats_ {
    ($enum_name:ident, $value:expr, $varname:ident, $function:expr) => {
        match $value {
            $enum_name::Le32($varname) => $function,
            $enum_name::Be32($varname) => $function,
            $enum_name::Le64($varname) => $function,
            $enum_name::Be64($varname) => $function,
        }
    };
}
