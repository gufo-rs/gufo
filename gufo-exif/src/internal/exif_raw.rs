mod debug;
mod decode;
mod edit;
mod lookup;
mod makernote;

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};

pub use gufo_common::exif::{Ifd, Tag, TagIfd};
use gufo_common::math::*;

pub use super::*;
use crate::error::{Error, Result, ResultExt};

#[derive(Debug, Clone, Copy)]
pub struct EntryRef {
    pub position: u32,
    pub data_type: Type,
    pub count: u32,
    pub value_offset: ValueOffset,
}

impl EntryRef {
    pub fn u32(&self) -> Option<u32> {
        if matches!(self.data_type, Type::Short | Type::Long) && self.count == 1 {
            if let ValueOffset::Value(value) = self.value_offset {
                return Some(value);
            }
        }

        None
    }

    pub fn value_offset_position(&self) -> u32 {
        self.position.safe_add(8).unwrap()
    }

    pub fn data_len(&self) -> Result<u32> {
        self.count
            .checked_mul(self.data_type.size())
            .e(Error::OffsetTooLarge)
    }

    pub fn offset(&self) -> Result<u32> {
        if let ValueOffset::Offset(offset) = self.value_offset {
            Ok(offset)
        } else {
            Err(Error::ValueInsteadOfOffset)
        }
    }
}

/// This can either be a value or an offset where to find the value
#[derive(Debug, Clone, Copy)]
pub enum ValueOffset {
    Value(u32),
    Offset(u32),
}

impl ValueOffset {
    fn new(data_type: Type, count: u32, value: u32) -> Result<Self> {
        let Some(size) = data_type.size().checked_mul(count) else {
            return Err(Error::DataSizeTooLarge);
        };
        Ok(if size <= 4 {
            Self::Value(value)
        } else {
            Self::Offset(value)
        })
    }

    fn u32(&self) -> u32 {
        match self {
            Self::Value(x) => *x,
            Self::Offset(x) => *x,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExifRaw {
    pub raw: Raw,
    pub locations: BTreeMap<TagIfd, Vec<EntryRef>>,
    /// The locations where the offsets are stored
    pub ifd_locations: BTreeMap<Ifd, u32>,
    pub makernote: bool,
}

impl ExifRaw {
    pub fn new(raw: Vec<u8>) -> Self {
        let raw = Raw {
            big_endian: false,
            buffer: Arc::new(Mutex::new(Cursor::new(raw))),
        };
        Self {
            raw,
            locations: Default::default(),
            ifd_locations: Default::default(),
            makernote: false,
        }
    }

    pub fn raw(&self) -> Raw {
        self.raw.clone()
    }
}
