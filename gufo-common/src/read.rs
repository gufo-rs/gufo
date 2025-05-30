use std::io::{Cursor, Seek};
use std::slice::SliceIndex;
use std::sync::Arc;

use crate::math::*;

pub trait GetExt<T> {
    fn e_get<I: SliceIndex<[T]>>(
        &self,
        index: I,
    ) -> Result<&<I as SliceIndex<[T]>>::Output, ReadError>;

    fn e_get_mut<I: SliceIndex<[T]>>(
        &mut self,
        index: I,
    ) -> Result<&mut <I as SliceIndex<[T]>>::Output, ReadError>;
}

impl<T> GetExt<T> for [T] {
    fn e_get<I: SliceIndex<[T]>>(
        &self,
        index: I,
    ) -> Result<&<I as SliceIndex<[T]>>::Output, ReadError> {
        self.get(index).ok_or(ReadError::InvalidIndex)
    }

    fn e_get_mut<I: SliceIndex<[T]>>(
        &mut self,
        index: I,
    ) -> Result<&mut <I as SliceIndex<[T]>>::Output, ReadError> {
        self.get_mut(index).ok_or(ReadError::InvalidIndex)
    }
}

pub trait ReadExt: std::io::BufRead + std::io::Seek {
    fn read_array<const T: usize>(&mut self) -> Result<[u8; T], ReadError> {
        let buf = &mut [0; T];
        self.read_exact(buf)?;
        Ok(*buf)
    }

    fn read_byte(&mut self) -> Result<u8, ReadError> {
        let buf = &mut [0; 1];
        self.read_exact(buf)?;
        Ok(buf[0])
    }
}

impl<T: AsRef<[u8]>> ReadExt for Cursor<T> {}

pub trait SliceExt<'a>: std::io::BufRead + std::io::Seek {
    fn slice_until(&mut self, byte: u8) -> Result<&'a [u8], ReadError>;
    fn slice_to_end(&mut self) -> Result<&'a [u8], ReadError>;
}

impl<'a> SliceExt<'a> for Cursor<&'a [u8]> {
    /// Read until `byte` and return as slice
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use gufo_common::read::*;
    /// let mut s = Cursor::new(b"abc\0defgh\0end".as_slice());
    /// assert_eq!(s.slice_until(b'\0').unwrap(), b"abc");
    /// assert_eq!(s.slice_until(b'\0').unwrap(), b"defgh");
    /// assert_eq!(s.slice_until(b'\0').unwrap(), b"end");
    /// ```
    fn slice_until(&mut self, byte: u8) -> Result<&'a [u8], ReadError> {
        let start = self.position().usize()?;
        let len = self
            .get_ref()
            .iter()
            .skip(start)
            .take_while(|x| **x != byte)
            .count();
        let end = start.safe_add(len)?;

        self.seek_relative(len.safe_add(1)?.i64()?)?;

        Ok(self.get_ref().get(start..end).unwrap())
    }

    /// Read until end and return as slice
    ///
    /// ```
    /// # use std::io::Cursor;
    /// # use gufo_common::read::*;
    /// let mut s = Cursor::new(b"abc\0end".as_slice());
    /// assert_eq!(s.slice_until(b'\0').unwrap(), b"abc");
    /// assert_eq!(s.slice_to_end().unwrap(), b"end");
    /// ```
    fn slice_to_end(&mut self) -> Result<&'a [u8], ReadError> {
        let start = self.position().usize()?;
        let end = self.get_ref().len();

        self.seek(std::io::SeekFrom::Start(end.u64()?))?;

        Ok(self.get_ref().get(start..end).unwrap())
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum ReadError {
    #[error("Math: {0}")]
    Math(#[from] MathError),
    #[error("IO: {0}")]
    Io(#[from] Arc<std::io::Error>),
    #[error("Invalid index")]
    InvalidIndex,
}

impl From<std::io::Error> for ReadError {
    fn from(value: std::io::Error) -> Self {
        Arc::new(value).into()
    }
}
