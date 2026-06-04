use std::collections::BTreeMap;

use gufo_common::exif::{IfdId, Tag};
use indexmap::IndexMap;
use zerocopy::{BigEndian, ByteOrder, IntoBytes, LittleEndian, U32, U64};

use super::util::IndexType;
use super::{Entry, EntryImmutable, EntryTyped};
use crate::error::Error;
use crate::structure::util::UsizeConversion;

#[derive(Debug)]
pub enum Ifd<'a> {
    Le32(IfdTyped<'a, U32<LittleEndian>, LittleEndian>),
    Be32(IfdTyped<'a, U32<BigEndian>, BigEndian>),
    Le64(IfdTyped<'a, U64<LittleEndian>, LittleEndian>),
    Be64(IfdTyped<'a, U64<BigEndian>, BigEndian>),
}

impl<'a> Ifd<'a> {
    pub fn serialize(&self) -> Vec<u8> {
        // Ifd entry list starts with number of entries
        let mut vec = crate::forall_formats!(self, ifd, ifd.n_entries.as_bytes()).to_vec();

        for entry in crate::forall_formats!(
            self,
            ifd,
            ifd.entries
                .values()
                .map(|x| x.serialize())
                .collect::<Vec<_>>()
        ) {
            vec.extend_from_slice(entry);
        }

        let ifd_offset = crate::forall_formats!(self, ifd, ifd.next_ifd_offset.as_bytes());
        vec.extend_from_slice(ifd_offset);

        vec
    }

    pub fn set_n_entries(&mut self, n_entires: usize) -> Result<(), Error> {
        crate::forall_formats!(self, ifd, ifd.set_n_entries(n_entires))
    }

    pub fn entry_by_tag(&mut self, tag: Tag) -> Option<Entry<'_>> {
        crate::forall_formats!(self, ifd, Some(ifd.entries.get_mut(&tag.0)?.as_entry()))
    }

    pub fn list_entry_relative_offset(&self, tag: Tag) -> Option<usize> {
        crate::forall_formats!(self, ifd, ifd.entries.get_index_of(&tag.0))
    }

    pub fn namespace(&self) -> IfdId {
        crate::forall_formats!(self, ifd, ifd.namespace)
    }

    pub fn entries_immutable(&mut self) -> Result<BTreeMap<u16, EntryImmutable>, Error> {
        crate::forall_formats!(
            self,
            ifd,
            ifd.entries
                .iter_mut()
                .map(|(k, v)| Ok((*k, v.as_entry().to_immutable()?)))
                .collect()
        )
    }

    pub const fn slist_length(&self) -> usize {
        crate::forall_formats!(
            self,
            ifd,
            ifd.n_entries.get() as usize * std::mem::size_of_val(&ifd.n_entries)
        )
    }

    pub fn n_entries(&self) -> usize {
        crate::forall_formats!(self, ifd, ifd.entries.len())
    }
}

#[derive(Debug)]
pub struct IfdTyped<'a, T: IndexType, O: ByteOrder> {
    pub namespace: IfdId,
    pub n_entries: &'a mut T::NEntries,
    pub entries: IndexMap<u16, &'a mut EntryTyped<T, O>>,
    pub next_ifd_offset: &'a mut T,
}

impl<'a, T: IndexType, O: ByteOrder> IfdTyped<'a, T, O> {
    fn set_n_entries(&mut self, n_entires: usize) -> Result<(), Error> {
        *self.n_entries = T::NEntries::try_from_usize(n_entires)?;

        Ok(())
    }
}
