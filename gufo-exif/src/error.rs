use std::array::TryFromSliceError;
use std::fmt::Display;
use std::num::TryFromIntError;
use std::ops::Range;

use gufo_common::math::MathError;
use zerocopy::ConvertError;

use crate::structure::Type;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("TryFromSlice")]
    TryFromSlice,
    #[error("TryFromInt")]
    TryFromInt,
    #[error("IndexUsed")]
    IndexUsed,
    #[error("IndexNotFound({0:?})")]
    IndexNotFound(Range<usize>),
    #[error("Alignment: {0}")]
    Alignment(String),
    #[error("IndexOverflow")]
    IndexOverflow,
    #[error("TryFroUnknownFormatmSlice")]
    UnknownFormat,
    #[error("TypeMissmatch: Expected one of '{1:?}', got '{0:?}'")]
    TypeMissmatch(Type, &'static [Type]),
    #[error("ElementCountMissmatch: Expected '{1}' elements, but got '{0}'")]
    ElementCountMissmatch(usize, usize),
    #[error("InputDataWrongLength: Expected data of length '{1}', got length '{0}'")]
    InputDataWrongLength(usize, usize),
    #[error("WouldIncreaseDataStore")]
    WouldIncreaseDataStore,
    #[error("MathError: {0}")]
    MathError(#[from] MathError),
    #[error("Other: {0}")]
    Other(String),
}

impl Error {
    pub fn other(s: impl Display) -> Self {
        Error::Other(s.to_string())
    }
}

impl From<TryFromSliceError> for Error {
    fn from(_value: TryFromSliceError) -> Self {
        Error::TryFromSlice
    }
}

impl From<TryFromIntError> for Error {
    fn from(_value: TryFromIntError) -> Self {
        Error::TryFromInt
    }
}

impl<A: Display, S: Display, V: Display> From<ConvertError<A, S, V>> for Error {
    fn from(err: ConvertError<A, S, V>) -> Self {
        Error::Alignment(format!("{err}"))
    }
}
