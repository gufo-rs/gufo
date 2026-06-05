mod high_level;

use std::marker::PhantomData;
use std::sync::Mutex;

use gufo_common::exif::TagIfd;
use zerocopy::FromZeros;

use crate::Error;
use crate::structure::{Document, Typed, ValueOrOffset};

/// Exif file
#[derive(Debug)]
pub struct Exif<'a, S: Storage<'a>> {
    document: S,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> Clone for Exif<'a, OwnedStore> {
    fn clone(&self) -> Self {
        Self::for_vec(self.serialize().unwrap()).unwrap()
    }
}

/// Data type on which [`Exif`] can be based
pub trait Storage<'a> {
    fn access<T>(&self, f: impl FnOnce(&mut Document) -> T) -> T;
}

#[ouroboros::self_referencing]
#[derive(Debug)]
pub struct OwnedStore {
    data: Vec<u8>,
    #[borrows(mut data)]
    #[not_covariant]
    document: Mutex<Document<'this>>,
}

impl<'a> Storage<'a> for OwnedStore {
    fn access<T>(&self, f: impl FnOnce(&mut Document) -> T) -> T {
        self.with_document(|x| f(&mut x.lock().unwrap()))
    }
}

pub struct MutBorrowedStore<'a> {
    document: Mutex<Document<'a>>,
}

impl<'a> Storage<'a> for MutBorrowedStore<'a> {
    fn access<T>(&self, f: impl FnOnce(&mut Document) -> T) -> T {
        f(&mut self.document.lock().unwrap())
    }
}

impl<'a> Exif<'a, OwnedStore> {
    /// Create from an owned vector
    pub fn for_vec(data: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            document: OwnedStore::try_new(data, |x| {
                Ok::<_, Error>(Mutex::new(Document::for_mut_slice(x)?))
            })?,
            lifetime: Default::default(),
        })
    }
}

impl<'a> Exif<'a, MutBorrowedStore<'a>> {
    /// Create for a mutable slice
    ///
    /// Functions that need to resize the underlying storage are not available
    /// with this constructor. This includes the deletion of entries.
    pub fn for_mut_slice(data: &'a mut [u8]) -> Result<Self, Error> {
        let document = Document::for_mut_slice(data)?;
        Ok(Self {
            document: MutBorrowedStore {
                document: Mutex::new(document),
            },
            lifetime: PhantomData::<&'a ()>,
        })
    }
}

impl<'a> Exif<'a, OwnedStore> {
    /// Delete entry
    ///
    /// The size of the raw exif data is not reduced. Deleted data is
    /// overwritten with zeros instead.
    pub fn delete(&mut self, tag_ifd: TagIfd) -> Result<bool, Error> {
        let Some((pos_retain_begin, pos_retain_end, pos_retain_new, pos_obsolete_retain_start)) =
            self.document(|document| {
                let entry_size = document.entry_size;
                let n_entries_size = document.n_entries_size;
                let index_size = document.index_size;

                let entry_data = if let Ok(Some(entry_data)) = document.entry_data(tag_ifd) {
                    // Fill data segment with zeros
                    entry_data.data.zero();
                    entry_data
                } else {
                    return Ok::<_, Error>(None);
                };

                // Number at which the entry lives in the ifd entry list
                let n_entry = entry_data.n_entry;

                let (ifd_pos, ifd) = document.ifd_pos(tag_ifd.ifd).unwrap();

                let ifd_n_entries = ifd.n_entries();
                let ifd_n_entries_behind_deleted = ifd_n_entries - n_entry;
                ifd.set_n_entries(ifd_n_entries - 1)?;

                let ifd_pos = *ifd_pos;

                // New position for entries we want to keep, this is the position of the deleted
                // entry in the list
                let pos_retain_new = ifd_pos + n_entries_size + n_entry * entry_size;
                // This is the range of list entries + next ifd after the deleted entry that we
                // want to keep
                let pos_retain_begin = pos_retain_new + entry_size;
                let pos_retain_end =
                    pos_retain_begin + (ifd_n_entries_behind_deleted - 1) * entry_size + index_size;
                // This is the start of the data that are duplicated after the copy
                let pos_obsolete_retain_start = pos_retain_end - entry_size;

                Ok(Some((
                    pos_retain_begin,
                    pos_retain_end,
                    pos_retain_new,
                    pos_obsolete_retain_start,
                )))
            })?
        else {
            return Ok(false);
        };

        // Get a raw exif data to edit them
        let mut raw = self.serialize()?;

        // Overwrite old list entry with remaining data
        raw.copy_within(pos_retain_begin..pos_retain_end, pos_retain_new);

        // Delete data that are now duplicated after the copy to not silently keep them
        // if the entry for them is deleted
        raw.get_mut(pos_obsolete_retain_start..pos_retain_end)
            .ok_or(Error::IndexOverflow)?
            .zero();

        // Parse exif again from raw data
        let exif = Exif::for_vec(raw)?;

        // Replace internal parsed data with new data
        self.document = exif.document;

        Ok(true)
    }
}

impl<'a, S: Storage<'a>> Exif<'a, S> {
    /// Overwrite stored entry
    ///
    /// The raw size of `value` can only be of the same size or smaller than the
    /// existing size. Otherwise, an [`Error::WouldIncreaseDataStore`] is
    /// returned.
    pub fn update_entry(&mut self, tag_ifd: TagIfd, value: Typed) -> Result<(), Error> {
        self.document(|document| {
            let data = value.serialize(document.endieness);
            let new_data_store = data.len() > document.index_size;

            let Some((_, mut entry)) = document.entry(tag_ifd) else {
                return Err(Error::other("Inserting new entries not yet supported."));
            };

            let old_data_len = entry.type_().size() * entry.count()?;

            let current_data_store = match entry.value_or_offset()? {
                ValueOrOffset::Value(_) => None,
                ValueOrOffset::Offset(offset) => Some(offset),
            };

            if current_data_store.is_none() && new_data_store {
                // Currentlt stored in ifd, but new needs data chunk
                return Err(Error::WouldIncreaseDataStore);
            } else if current_data_store.is_none() && !new_data_store {
                // Data remains in ifd
                entry.update(tag_ifd.tag, value.type_(), value.count(), data)?;
            } else if let Some(old_data_offset) = current_data_store
                && !new_data_store
            {
                // Currently stored in data, but new in ifd
                let old_data_offset_end = old_data_offset + old_data_len;

                entry.update_offset(tag_ifd.tag, value.type_(), value.count(), old_data_offset)?;

                document
                    .data(old_data_offset..old_data_offset_end)
                    .ok_or(Error::other(
                        "Expected to find data to delete for {tag_ifd:?}",
                    ))?
                    .zero();
            } else if let Some(old_data_offset) = current_data_store
                && new_data_store
            {
                if data.len() > old_data_len {
                    return Err(Error::WouldIncreaseDataStore);
                }

                // Data remains in in store
                let old_data_offset_end = old_data_offset + old_data_len;

                entry.update_offset(tag_ifd.tag, value.type_(), value.count(), old_data_offset)?;

                let x = document
                    .data(old_data_offset..old_data_offset_end)
                    .ok_or(Error::other(
                        "Expected to find data to delete for {tag_ifd:?}",
                    ))?;
                x.zero();

                x.get_mut(..data.len())
                    .ok_or(Error::IndexOverflow)?
                    .copy_from_slice(&data);
            }

            Ok::<_, Error>(())
        })
    }

    /// Bytes that change by an update
    ///
    /// Identical to [`update_entry`](Self::update_entry). Aditionally, a list
    /// of bytes with their corresponding values is returned. Changing these
    /// bytes in the original raw exif would achieve the requested update.
    pub fn update_entry_diff(
        &mut self,
        tag_ifd: TagIfd,
        value: Typed,
    ) -> Result<Vec<(usize, u8)>, Error> {
        let serialized_old = self.serialize()?;
        self.update_entry(tag_ifd, value)?;
        let serialized_new = self.serialize()?;

        let mut changes = Vec::new();
        for (n_byte, (old, new)) in serialized_old.into_iter().zip(serialized_new).enumerate() {
            if old != new {
                changes.push((n_byte, new));
            }
        }

        Ok(changes)
    }
}
