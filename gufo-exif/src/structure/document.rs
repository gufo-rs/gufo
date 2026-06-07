mod high_level;
mod type_lookup;

use std::collections::BTreeMap;
use std::ops::Range;

use gufo_common::exif::{IfdId, TagIfd};

use super::Ifd;
use crate::Error;
use crate::structure::util::Endieness;
use crate::structure::{Entry, Parser, Type, Typed, ValueOrOffset};

/// Exif Document
///
/// Access to the lower lying structures of an Exif document.
#[derive(Debug)]
pub struct Document<'a> {
    ifds: BTreeMap<IfdId, (usize, Ifd<'a>)>,
    data: Vec<(usize, &'a mut [u8])>,
    pub(crate) endieness: Endieness,
    primary_ifd_offset: &'a mut [u8],
    pub(crate) index_size: usize,
    pub(crate) n_entries_size: usize,
    pub(crate) entry_size: usize,
}

pub struct EntryTyped {
    pub tag_ifd: TagIfd,
    pub count: usize,
    pub type_: Type,
    pub data: Result<Typed, Error>,
}

pub struct EntryData<'a> {
    pub n_entry: usize,
    pub count: usize,
    pub type_: Type,
    pub data: &'a mut [u8],
}

impl<'a> Document<'a> {
    pub fn for_mut_slice(data: &'a mut [u8]) -> Result<Self, Error> {
        let mut file_parser = Parser::new(data)?;

        let ifds = file_parser.parse()?;
        let endieness = file_parser.endieness();
        let index_size = file_parser.index_size();
        let n_entries_size = file_parser.n_entries_size();
        let entry_size = file_parser.entry_size();

        let (primary_ifd_offset, data) = file_parser.data();

        Ok(Self {
            ifds,
            data,
            endieness,
            primary_ifd_offset,
            index_size,
            n_entries_size,
            entry_size,
        })
    }

    pub fn serialize(&mut self) -> Result<Vec<u8>, Error> {
        let mut data = self
            .data
            .iter()
            .map(|(pos, data)| (*pos, data.to_vec()))
            .collect::<Vec<_>>();

        data.push((4, self.primary_ifd_offset.to_vec()));

        let ifd_data = self
            .ifds()
            .values()
            .map(|(pos, ifd)| (*pos, ifd.serialize()))
            .collect::<Vec<_>>();

        data.extend(ifd_data);

        data.sort_by_key(|(pos, _)| *pos);

        let mut vec = Vec::new();
        for (pos, fragment) in data {
            if vec.len() != pos {
                return Err(Error::other(format!(
                    "Document fragments do not fit together. Want to insert at {pos} but at currently at {}",
                    vec.len()
                )));
            }
            vec.extend(fragment);
        }

        Ok(vec)
    }

    pub fn entries(
        &mut self,
    ) -> Result<BTreeMap<IfdId, BTreeMap<gufo_common::exif::Tag, EntryTyped>>, Error> {
        let mut entries = Vec::new();

        for (ifd_id, (_, list)) in self.ifds.iter_mut() {
            for entry in list.entries()?.values() {
                let tag_ifd = TagIfd::new(entry.tag(), *ifd_id);

                entries.push((tag_ifd, entry.count()?, entry.type_()));
            }
        }

        let mut xs = BTreeMap::new();

        for (tag_ifd, count, type_) in entries {
            let y = xs.entry(tag_ifd.ifd).or_insert_with(BTreeMap::new);

            let data = self
                .lookup(tag_ifd)
                .transpose()
                .ok_or_else(|| {
                    Error::Other(format!(
                        "Couldn't find tag_ifd, but it should haven been there: {tag_ifd:?}."
                    ))
                })
                .flatten();

            let x = EntryTyped {
                tag_ifd,
                count,
                type_,
                data,
            };

            y.insert(tag_ifd.tag, x);
        }

        Ok(xs)
    }

    pub fn list_entry_offset(&self, tag_ifd: TagIfd) -> Option<usize> {
        let (ifd_offset, ifd) = self.ifds.get(&tag_ifd.ifd)?;

        let entry_relative_offset = ifd.list_entry_relative_offset(tag_ifd.tag)?;

        Some(ifd_offset + entry_relative_offset)
    }

    pub fn ifd(&mut self, ifd: IfdId) -> Option<&mut Ifd<'a>> {
        Some(&mut self.ifds.get_mut(&ifd)?.1)
    }

    pub fn ifd_pos(&mut self, ifd: IfdId) -> Option<&mut (usize, Ifd<'a>)> {
        self.ifds.get_mut(&ifd)
    }

    pub fn entry(&mut self, tag_ifd: TagIfd) -> Option<(usize, Entry<'_>)> {
        let ifd: &mut Ifd<'a> = self.ifd(tag_ifd.ifd)?;
        crate::forall_formats!(Ifd, ifd, ifd, {
            let entry = ifd.entries.get_full_mut(&tag_ifd.tag.0)?;
            Some((entry.0, entry.2.as_entry()))
        })
    }

    pub fn entry_data(&mut self, tag_ifd: TagIfd) -> Result<Option<EntryData<'_>>, Error> {
        let Some(ifd) = self.ifd(tag_ifd.ifd) else {
            return Ok(None);
        };

        let Some((n_entry, type_, count, value_or_offset)) = crate::forall_formats!(
            Ifd,
            ifd,
            ifd,
            ifd.entries
                .get_full_mut(&tag_ifd.tag.0)
                .map(|(n_entry, _, x)| (n_entry, x.type_(), x.count(), x.value_or_offset()))
        ) else {
            return Ok(None);
        };

        let value_or_offset = value_or_offset?;
        let count = count?;

        let offset = if let ValueOrOffset::Offset(offset) = value_or_offset {
            Some(offset)
        } else {
            None
        };

        let data = if let Some(offset) = offset {
            let len = count * type_.size();
            let data_range = offset..offset + len;
            self.data(data_range.clone())
                .ok_or(Error::IndexNotFound(data_range))?
        } else {
            let Some(ifd) = self.ifd(tag_ifd.ifd) else {
                return Ok(None);
            };

            let value_or_offset = crate::forall_formats!(
                Ifd,
                ifd,
                ifd,
                ifd.entries
                    .get_mut(&tag_ifd.tag.0)
                    .map(|x| x.value_or_offset())
                    .transpose()?
            );

            let Some(value_or_offset) = value_or_offset else {
                return Ok(None);
            };

            if let ValueOrOffset::Value(value) = value_or_offset {
                value
            } else {
                unreachable!()
            }
        };

        Ok(Some(EntryData {
            n_entry,
            type_,
            count,
            data,
        }))
    }

    pub fn data(&mut self, range: Range<usize>) -> Option<&mut [u8]> {
        self.data
            .iter_mut()
            .find(|(p, x)| *p <= range.start && range.end <= p + x.len())
            .and_then(|(p, x)| x.get_mut(range.start - *p..range.end - *p))
    }

    pub fn data_blocks(&mut self) -> &[(usize, &mut [u8])] {
        &self.data
    }

    pub fn ifds(&mut self) -> &mut BTreeMap<IfdId, (usize, Ifd<'a>)> {
        &mut self.ifds
    }
}
