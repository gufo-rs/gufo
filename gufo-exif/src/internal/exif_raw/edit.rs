use super::*;

impl super::ExifRaw {
    pub fn set_existing(&mut self, tagifd: impl Into<TagIfd>, value: u32) -> Result<()> {
        let tagifd = tagifd.into();
        let mut raw = self.raw();
        let entries = self.locations.entry(tagifd).or_default();

        if entries.is_empty() {
            Err(Error::TagNotFound(tagifd))
        } else {
            for entry in entries {
                raw.seek_start(entry.position.safe_add(8)?)?;
                entry.value_offset = ValueOffset::new(Type::Long, entry.count, value)?;
                raw.write_u32(value)?;
            }
            Ok(())
        }
    }

    pub fn insert_entry(&mut self, tagifd: impl Into<TagIfd>, value: EntryRef) -> Result<()> {
        let tagifd = tagifd.into();
        let mut raw = self.raw();

        let Some(ifd_location) = self.ifd_locations.get(&tagifd.ifd).copied() else {
            // TODO: Create IFD instead
            return Err(Error::IfdNotFound);
        };

        self.raw().seek_start(ifd_location)?;
        let ifd_offset = self.raw().read_u32()?;
        self.raw().seek_start(ifd_offset)?;

        let n_entries_position = self.raw().position()?;
        let n_entries = self.raw().read_u16()?;

        let entries_end = self.raw().position()?.safe_add(u32::from(n_entries) * 12)?;
        let entries_index: usize = entries_end.try_into().map_err(|_| Error::OffsetTooLarge)?;

        self.makernote_handle_insert(entries_end, 12)?;

        // Insert space for new entry
        self.add_empty_space(entries_index, 12)?;
        self.inserted_at(entries_end, 12)?;

        // Update number of entries
        raw.seek_start(n_entries_position)?;
        raw.write_u16(n_entries.checked_add(1).e(Error::AdditionOverflow)?)?;

        // Write new entry
        raw.seek_start(entries_end)?;
        raw.write_u16(tagifd.tag.0)?;
        raw.write_u16(value.data_type.u16())?;
        raw.write_u32(value.count)?;
        raw.write_u32(value.value_offset.u32())?;

        // Update internal table
        raw.seek_start(entries_end)?;
        let (_, entry) = self.read_entry()?;
        self.locations.entry(tagifd).or_default().push(entry);

        Ok(())
    }

    fn add_empty_space(&mut self, index: usize, len: usize) -> Result<()> {
        self.raw
            .buffer
            .borrow_mut()
            .get_mut()
            .splice(index..index, vec![0; len]);
        Ok(())
    }

    pub fn inserted_at(&mut self, insert_position: u32, len: i64) -> Result<()> {
        let mut raw = self.raw();

        // Shift all existing offsets in entries
        for vec in self.locations.values_mut() {
            for entry in vec {
                let new_position = if entry.position >= insert_position {
                    (entry.position.i64().safe_add(len)?).u32()?
                } else {
                    entry.position
                };
                entry.position = new_position;
                if let ValueOffset::Offset(offset) = &mut entry.value_offset {
                    if *offset >= insert_position {
                        let new_offset = (offset.i64().safe_add(len)?).u32()?;
                        *offset = new_offset;
                        raw.seek_start(new_position.safe_add(8)?)?;
                        raw.write_u32(new_offset)?;
                    }
                }
            }
        }

        // Shift ifd offset
        for location in self.ifd_locations.values_mut() {
            let offset_location = if *location >= insert_position {
                (location.i64().safe_add(len)?).u32()?
            } else {
                *location
            };

            raw.seek_start(offset_location)?;
            let current_offset = raw.read_u32()?;

            if current_offset >= insert_position {
                raw.seek_start(offset_location)?;
                raw.write_u32((current_offset.i64().safe_add(len)?).u32()?)?;
                *location = offset_location;
            }
        }

        Ok(())
    }
}
