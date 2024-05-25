use std::cell::RefCell;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

use crate::error::{Error, Result, ResultExt};

#[derive(Debug, Clone)]
pub struct Raw {
    pub big_endian: bool,
    pub buffer: Rc<RefCell<Cursor<Vec<u8>>>>,
}

impl Raw {
    pub fn position(&self) -> Result<u32> {
        self.buffer
            .borrow()
            .position()
            .try_into()
            .e(Error::OffsetTooLarge)
    }

    pub fn seek_start(&mut self, seek: u32) -> Result<()> {
        self.buffer
            .borrow_mut()
            .seek(SeekFrom::Start(seek.into()))?;

        Ok(())
    }

    pub fn read_exact<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut bytes: [u8; N] = [0; N];
        self.buffer.borrow_mut().read_exact(&mut bytes)?;
        Ok(bytes)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_exact()?;
        Ok(if self.big_endian {
            u16::from_be_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        })
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_exact()?;
        Ok(if self.big_endian {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        })
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.buffer
            .borrow_mut()
            .write_all(bytes)
            .map_err(Into::into)
    }

    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        let bytes = if self.big_endian {
            u16::to_be_bytes(value)
        } else {
            u16::to_le_bytes(value)
        };

        self.write_all(&bytes)?;

        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        let bytes = if self.big_endian {
            u32::to_be_bytes(value)
        } else {
            u32::to_le_bytes(value)
        };

        self.write_all(&bytes)?;

        Ok(())
    }
}
