use gufo_common::utils::{
    AdditionOverflowError, ConversionOverflowError, SubstractionOverflowError,
};

use crate::internal::{Ifd, TagIfd, Type};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnkownByteOrder([u8; 2]),
    MagicBytesWrong(u16),
    Io(std::io::Error),
    TagNotFound(TagIfd),
    IfdShouldTerminate(Ifd),
    OffsetTooLarge,
    LookupEof,
    ByteOrderEof,
    MagicBytesEof,
    EntryEof,
    NumerEntriesEof,
    InvalidLookupOffset,
    DataSizeTooLarge,
    IfdNotFound,
    WrongTypeGeneric,
    WrongType {
        expected: (u32, Type),
        actual: (u32, Type),
    },
    OffsetInvalid(i64),
    OffsetInsteadOfValue,
    ValueInsteadOfOffset,
    IncompatibleValue,
    AdditionOverflow,
    SubstractionOverflowError,
    ConversionOverflowError,
    EntryNotFound,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<AdditionOverflowError> for Error {
    fn from(_: AdditionOverflowError) -> Self {
        Self::AdditionOverflow
    }
}

impl From<SubstractionOverflowError> for Error {
    fn from(_: SubstractionOverflowError) -> Self {
        Self::SubstractionOverflowError
    }
}

impl From<ConversionOverflowError> for Error {
    fn from(_: ConversionOverflowError) -> Self {
        Self::ConversionOverflowError
    }
}

pub(crate) trait ResultExt<T> {
    fn e(self, err: Error) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E> {
    fn e(self, err: Error) -> Result<T> {
        self.map_err(|_| err)
    }
}

impl<T> ResultExt<T> for Option<T> {
    fn e(self, err: Error) -> Result<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(err),
        }
    }
}
