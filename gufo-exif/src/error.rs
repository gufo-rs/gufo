use std::sync::Arc;

use gufo_common::utils::{
    AdditionOverflowError, ConversionOverflowError, SubstractionOverflowError,
};

use crate::internal::{Ifd, TagIfd, Type};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Unkown byte order: {0:x?}")]
    UnkownByteOrder([u8; 2]),
    #[error("Wrong magic bytes: {0:x?}")]
    MagicBytesWrong(u16),
    #[error("IO error: {0}")]
    Io(Arc<std::io::Error>),
    #[error("Tag not found: {0:?}")]
    TagNotFound(TagIfd),
    #[error("Ifd should terminate: {0:?}")]
    IfdShouldTerminate(Ifd),
    #[error("OffsetTooLarge")]
    OffsetTooLarge,
    #[error("LookupEof")]
    LookupEof,
    #[error("LookupEof")]
    ByteOrderEof,
    #[error("ByteOrderEof")]
    MagicBytesEof,
    #[error("MagicBytesEof")]
    EntryEof,
    #[error("EntryEof")]
    NumerEntriesEof,
    #[error("NumerEntriesEof")]
    InvalidLookupOffset,
    #[error("InvalidLookupOffset")]
    DataSizeTooLarge,
    #[error("DataSizeTooLarge")]
    IfdNotFound,
    #[error("IfdNotFound")]
    WrongTypeGeneric,
    #[error("WrongTypeGeneric")]
    WrongType {
        expected: (u32, Type),
        actual: (u32, Type),
    },
    #[error("OffsetInvalid: {0}")]
    OffsetInvalid(i64),
    #[error("OffsetInsteadOfValue")]
    OffsetInsteadOfValue,
    #[error("ValueInsteadOfOffset")]
    ValueInsteadOfOffset,
    #[error("IncompatibleValue")]
    IncompatibleValue,
    #[error("AdditionOverflow")]
    AdditionOverflow,
    #[error("SubstractionOverflowError")]
    SubstractionOverflowError,
    #[error("ConversionOverflowError")]
    ConversionOverflowError,
    #[error("EntryNotFound")]
    EntryNotFound,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(Arc::new(value))
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
