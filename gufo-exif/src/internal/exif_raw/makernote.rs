use core::panic;
use std::ops::Range;

use gufo_common::math::U32Ext;

use super::{EntryRef, Ifd, Tag, TagIfd, ValueOffset};
use crate::error::{Error, Result, ResultExt};

impl super::ExifRaw {
    pub fn makernote_entry(&mut self) -> Option<EntryRef> {
        self.lookup_entry(TagIfd::new(Tag::MAKER_NOTE, Ifd::Exif))
    }

    pub fn makernote_handle_insert(&mut self, insert_position: u32, len: u32) -> Result<()> {
        let Some(makernote_entry) = self.makernote_entry() else {
            return Ok(());
        };
        if let ValueOffset::Offset(makernote_offset) = makernote_entry.value_offset {
            if makernote_offset >= insert_position {
                self.freeup_space_before(
                    makernote_offset,
                    len,
                    makernote_offset.safe_add(makernote_entry.data_len()?)?,
                )?;

                let last_position = self.last_data_end_before(makernote_offset)?;

                self.raw
                    .buffer
                    .borrow_mut()
                    .get_mut()
                    .drain(last_position.usize()..last_position.safe_add(len)?.usize());
                self.inserted_at(last_position, -len.i64())?;
            }
        }

        Ok(())
    }

    pub fn last_data_end_before(&self, offset: u32) -> Result<u32> {
        let last_entry = self
            .locations
            .values()
            .flat_map(|x| x.iter())
            .filter_map(|x| x.offset().ok().map(|i| (i, x)))
            .filter(|(i, _)| *i < offset)
            .max_by_key(|(i, _)| *i);

        if let Some((last_offset, entry)) = last_entry {
            last_offset.safe_add(entry.data_len()?).map_err(Into::into)
        } else {
            Ok(0)
        }
    }

    pub fn free_space_before(&self, offset: u32) -> Result<u32> {
        let end = self.last_data_end_before(offset)?;

        offset.safe_sub(end).map_err(Into::into)
    }

    /// Tries to free up `len` space
    pub fn freeup_space_before(
        &mut self,
        before_position: u32,
        len: u32,
        post_here: u32,
    ) -> Result<()> {
        // TODO: Check overflow
        let mut candidates: Vec<_> = self
            .locations
            .values()
            .flat_map(|v| v.iter())
            .filter_map(|x| match x.value_offset {
                ValueOffset::Offset(offset) if offset < before_position => Some((offset, *x)),
                _ => None,
            })
            .collect();

        candidates.sort_by_key(|x| x.0);

        let mut freed = 0;
        let mut to_move = Vec::new();

        while let Some((_, entry)) = candidates.pop() {
            freed = freed.safe_add(entry.data_len()?)?;
            to_move.push(entry);
            if freed >= len {
                break;
            }
        }

        if freed >= len {
            for entry in to_move {
                self.move_entry(entry, post_here, false)?;
            }
        } else {
            panic!("ahhhh");
        }

        Ok(())
    }

    fn move_entry(
        &mut self,
        entry: EntryRef,
        new_position_u32: u32,
        overwrite: bool,
    ) -> Result<()> {
        let mut raw = self.raw();

        let entry = self
            .locations
            .values_mut()
            .flat_map(|x| x.iter_mut())
            .find(|x| x.position == entry.position)
            .e(Error::EntryNotFound)?;

        let range = if let ValueOffset::Offset(offset) = &mut entry.value_offset {
            let old_offset = *offset;
            *offset = new_position_u32;

            old_offset..old_offset.safe_add(entry.data_len()?)?
        } else {
            return Err(Error::EntryNotFound);
        };

        // Update raw entry with new offset
        raw.seek_start(entry.value_offset_position())?;
        raw.write_u32(new_position_u32)?;

        self.move_datum(range, new_position_u32, overwrite)?;

        Ok(())
    }

    fn move_datum(
        &mut self,
        range: Range<u32>,
        new_position_u32: u32,
        overwrite: bool,
    ) -> Result<()> {
        let new_position = new_position_u32.usize();
        let len = range.end.safe_sub(range.start)?;
        let len_usize = len.usize();

        let old_range = range.start.usize()..range.end.usize();
        let new_range = new_position
            ..(new_position
                .checked_add(len_usize)
                .e(Error::OffsetTooLarge)?);

        let data = self
            .raw
            .buffer
            .borrow_mut()
            .get_ref()
            .get(old_range.clone())
            .e(Error::EntryEof)?
            .to_vec();

        // Overwrite old data with zeros
        self.raw
            .buffer
            .borrow_mut()
            .get_mut()
            .get_mut(old_range)
            .e(Error::EntryEof)?
            .copy_from_slice(&vec![0; len_usize]);

        if overwrite {
            self.raw
                .buffer
                .borrow_mut()
                .get_mut()
                .get_mut(new_range)
                .e(Error::EntryEof)?
                .copy_from_slice(&data);
        } else {
            self.raw
                .buffer
                .borrow_mut()
                .get_mut()
                .splice(new_position..new_position, data);
        }

        if !overwrite {
            self.inserted_at(new_position_u32.safe_add(len)?, len.i64())?;
        }

        Ok(())
    }

    pub fn makernote_register(&mut self) -> Result<()> {
        if let Some(entry) = self.makernote_entry() {
            if let ValueOffset::Offset(offset) = entry.value_offset {
                if let Some(internal_offset) = self.makernote_guess_offset() {
                    let offset_location = entry.value_offset_position();
                    self.add_ifd_offset(Ifd::MakerNote, offset_location);

                    self.raw().seek_start(offset.safe_add(internal_offset)?)?;
                    self.decode_entries(Ifd::MakerNote)?;

                    if self.validate_ifd(Ifd::MakerNote).is_ok() {
                        self.makernote = true;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn makernote_guess_offset(&mut self) -> Option<u32> {
        if let Some(entry) = self.makernote_entry() {
            if let ValueOffset::Offset(offset) = entry.value_offset {
                self.raw().seek_start(offset).ok()?;
                let data = self.raw().read_exact::<20>().ok()?;
                if data.starts_with(b"Apple iOS") {
                    return Some(14);
                }

                return Some(0);
            }
        }

        None
    }

    pub fn validate_ifd(&mut self, ifd: Ifd) -> Result<()> {
        let tags = self
            .locations
            .keys()
            .filter(|tagifd| tagifd.ifd == ifd)
            .cloned()
            .collect::<Vec<_>>();

        for tagifd in tags {
            self.lookup_data(tagifd)?;
        }

        Ok(())
    }
}
