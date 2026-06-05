use std::collections::BTreeMap;
use std::marker::PhantomData;

use gufo_common::exif::{Field, IfdId};
use indexmap::IndexMap;
use zerocopy::{BigEndian, ByteOrder, FromBytes, LittleEndian, U16, U32, U64};

use super::util::{IndexType, UsizeConversion};
use super::{Ifd, IfdGeneric};
use crate::error::Error;
use crate::structure::EntryGeneric;
use crate::structure::util::{Endieness, handle_error_};

const MAGIC_BYTES_LE_32: &[u8] = b"II*\0";
const MAGIC_BYTES_BE_32: &[u8] = b"MM\0*";
const MAGIC_BYTES_LE_64: &[u8] = b"II+\0";
const MAGIC_BYTES_BE_64: &[u8] = b"MM\0+";

#[derive(Debug)]
pub(crate) enum Parser<'a> {
    Le32(ParserGeneric<'a, U32<LittleEndian>, LittleEndian>),
    Be32(ParserGeneric<'a, U32<BigEndian>, BigEndian>),
    Le64(ParserGeneric<'a, U64<LittleEndian>, LittleEndian>),
    Be64(ParserGeneric<'a, U64<BigEndian>, BigEndian>),
}

impl<'a> Parser<'a> {
    pub fn new(data: &'a mut [u8]) -> Result<Self, Error> {
        let magic_bytes = data.get(..4).ok_or(Error::TryFromSlice)?;

        Ok(match magic_bytes {
            MAGIC_BYTES_BE_32 => Self::Be32(ParserGeneric::<U32<BigEndian>, BigEndian>::new(data)),
            MAGIC_BYTES_LE_32 => {
                Self::Le32(ParserGeneric::<U32<LittleEndian>, LittleEndian>::new(data))
            }
            MAGIC_BYTES_BE_64 => {
                let mut x = Self::Be64(ParserGeneric::<U64<BigEndian>, BigEndian>::new(data));
                x.read_u16()?;
                x.read_u16()?;
                x
            }
            MAGIC_BYTES_LE_64 => {
                let mut x = Self::Le64(ParserGeneric::<U64<LittleEndian>, LittleEndian>::new(data));
                x.read_u16()?;
                x.read_u16()?;
                x
            }
            _ => return Err(Error::UnknownFormat),
        })
    }

    pub fn parse(&mut self) -> Result<BTreeMap<IfdId, (usize, Ifd<'a>)>, Error> {
        // Record magic bytes into data
        self.seek_absolute(4)?;

        let mut ifds = BTreeMap::new();

        // Read the initial offset in the file to the first Ifd
        let primary_ifd_offset = self.read_primary_ifd_offset()?;
        self.seek_absolute(primary_ifd_offset)?;

        let mut primary_ifd = self.read_ifd(IfdId::Primary)?;

        // Read Exif Ifd if available
        if let Some(mut exif_ifd_pointer) =
            primary_ifd.entry_by_tag(gufo_common::field::ExifIFDPointer::TAG)
        {
            if let Some(offset) = handle_error_(exif_ifd_pointer.ifd_pointer()).map(|x| x as usize)
            {
                self.seek_absolute(offset)?;
                let mut exif_ifd = self.read_ifd(IfdId::Exif)?;

                // Read Maker Info Ifd if available
                if let Some(mut maker_ifd_pointer) =
                    exif_ifd.entry_by_tag(gufo_common::field::MakerNote::TAG)
                {
                    if let Some(offset) =
                        handle_error_(maker_ifd_pointer.ifd_pointer()).map(|x| x as usize)
                    {
                        self.seek_absolute(offset)?;
                        let maker_info_ifd = self.read_ifd(IfdId::Gps)?;
                        ifds.insert(IfdId::MakerNote, (offset, maker_info_ifd));
                    }
                }

                ifds.insert(IfdId::Exif, (offset, exif_ifd));
            }
        }

        // Read GPS Info Ifd if available
        if let Some(mut gps_ifd_pointer) =
            primary_ifd.entry_by_tag(gufo_common::field::GPSInfoIFDPointer::TAG)
        {
            if let Some(offset) = handle_error_(gps_ifd_pointer.ifd_pointer()).map(|x| x as usize) {
                self.seek_absolute(offset)?;
                let gps_info_ifd = self.read_ifd(IfdId::Gps)?;
                ifds.insert(IfdId::Gps, (offset, gps_info_ifd));
            }
        }

        ifds.insert(IfdId::Primary, (primary_ifd_offset, primary_ifd));

        // Add remaining data in document since it can contain data referenced from
        // entrties
        self.read_remaining_data();

        Ok(ifds)
    }

    pub fn n_entries_size(&self) -> usize {
        crate::forall_formats_self!(self, file, file.n_entries_size())
    }

    pub fn index_size(&self) -> usize {
        crate::forall_formats_self!(self, file, file.index_size())
    }

    pub fn entry_size(&self) -> usize {
        crate::forall_formats_self!(self, file, file.entry_size())
    }

    pub fn read_primary_ifd_offset(&mut self) -> Result<usize, Error> {
        crate::forall_formats_self!(self, file, file.read_primary_ifd_offset())
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        crate::forall_formats_self!(self, file, Ok(file.read_u16()?.get()))
    }

    pub fn seek_absolute(&mut self, abs_pos: usize) -> Result<(), Error> {
        crate::forall_formats_self!(self, file, file.seek_absolute(abs_pos))
    }

    pub fn read_ifd(&mut self, ifd: IfdId) -> Result<Ifd<'a>, Error> {
        Ok(match self {
            Parser::Be32(x) => Ifd::Be32(x.read_ifd(ifd)?),
            Parser::Le32(x) => Ifd::Le32(x.read_ifd(ifd)?),
            Parser::Be64(x) => Ifd::Be64(x.read_ifd(ifd)?),
            Parser::Le64(x) => Ifd::Le64(x.read_ifd(ifd)?),
        })
    }

    pub fn data(self) -> (&'a mut [u8], Vec<(usize, &'a mut [u8])>) {
        crate::forall_formats_self!(self, file, (file.primary_ifd_offset, file.data))
    }

    pub fn endieness(&self) -> Endieness {
        match self {
            Self::Be32(_) | Self::Be64(_) => Endieness::Big,
            Self::Le32(_) | Self::Le64(_) => Endieness::Litte,
        }
    }

    pub fn read_remaining_data(&mut self) {
        crate::forall_formats_self!(self, file, file.read_remaining_data())
    }
}

#[derive(Debug)]
pub(crate) struct ParserGeneric<'a, T, O> {
    remaining_data: &'a mut [u8],
    pointer_type: PhantomData<T>,
    endieness: PhantomData<O>,
    pos: usize,
    primary_ifd_offset: &'a mut [u8],
    data: Vec<(usize, &'a mut [u8])>,
}

impl<'a, T: IndexType, O: ByteOrder> ParserGeneric<'a, T, O> {
    fn new(remaining_data: &'a mut [u8]) -> Self {
        Self {
            remaining_data,
            pos: 0,
            pointer_type: Default::default(),
            endieness: Default::default(),
            primary_ifd_offset: Default::default(),
            data: Default::default(),
        }
    }

    /// Read specified number of bytes
    fn read_bytes(&mut self, n_bytes: usize) -> Result<&'a mut [u8], Error> {
        let current_data = std::mem::take(&mut self.remaining_data);

        let (x, y) = current_data
            .split_at_mut_checked(n_bytes)
            .ok_or(Error::IndexOverflow)?;

        self.remaining_data = y;
        self.pos = self.pos.checked_add(n_bytes).ok_or(Error::IndexOverflow)?;

        Ok(x)
    }

    /// Read index value, 32 bit for TIFF, 64 bit for BigTIFF
    fn read_index(&mut self) -> Result<&'a mut T, Error> {
        let bytes = self.read_bytes(std::mem::size_of::<T>())?;

        T::mut_from_bytes(bytes).map_err(Into::into)
    }

    fn read_primary_ifd_offset(&mut self) -> Result<usize, Error> {
        let index = self.read_index()?;
        let index_usize = index.try_to_usize()?;
        self.primary_ifd_offset = index.as_mut_bytes();

        Ok(index_usize)
    }

    fn read_u16(&mut self) -> Result<&'a mut U16<O>, Error> {
        let bytes = self.read_bytes(2)?;

        U16::<O>::mut_from_bytes(bytes).map_err(Into::into)
    }

    fn seek_absolute(&mut self, abs_pos: usize) -> Result<(), Error> {
        let pos = self.pos;
        let relative_position = abs_pos.checked_sub(self.pos).unwrap();

        let bytes = self.read_bytes(relative_position)?;

        if !bytes.is_empty() {
            self.data.push((pos, bytes));
        }

        Ok(())
    }

    /// Read number of entries in an ifd
    ///
    /// This number is located directly before the list of entries.
    fn read_n_entries(&mut self) -> Result<&'a mut T::NEntries, Error> {
        let bytes = self.read_bytes(std::mem::size_of::<T::NEntries>())?;
        T::NEntries::mut_from_bytes(bytes).map_err(Into::into)
    }

    /// Read one entry from the ifd
    fn read_entry(&mut self) -> Result<&'a mut EntryGeneric<T, O>, Error> {
        let size = self.entry_size();

        let bytes = self.read_bytes(size)?;

        EntryGeneric::mut_from_bytes(bytes).map_err(Into::into)
    }

    /// Read entries from ifd
    fn read_ifd(&mut self, ifd: IfdId) -> Result<IfdGeneric<'a, T, O>, Error> {
        let n_entries = self.read_n_entries()?;
        let mut entries = IndexMap::new();
        for _ in 0..n_entries.try_to_usize()? {
            let entry = self.read_entry()?;
            entries.insert(entry.tag_id.get(), entry);
        }

        let ifd_offset = self.read_index()?;

        Ok(IfdGeneric {
            namespace: ifd,
            n_entries,
            entries,
            next_ifd_offset: ifd_offset,
        })
    }

    fn read_remaining_data(&mut self) {
        let remaining_data = std::mem::take(&mut self.remaining_data);
        let pos = self.pos;
        self.pos += remaining_data.len();

        self.data.push((pos, remaining_data));
    }

    fn n_entries_size(&self) -> usize {
        std::mem::size_of::<T::NEntries>()
    }

    fn index_size(&self) -> usize {
        std::mem::size_of::<T>()
    }

    /// Byte size of one entry in the table
    fn entry_size(&self) -> usize {
        std::mem::size_of::<EntryGeneric<T, O>>()
    }
}
