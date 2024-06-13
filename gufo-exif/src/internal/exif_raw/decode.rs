use super::*;

impl super::ExifRaw {
    /// Decode data in buffer
    ///
    /// See 4.5.2 in v3.0 standard
    pub fn decode(&mut self) -> Result<()> {
        self.locations = Default::default();
        self.ifd_locations = Default::default();

        self.decode_header()?;

        self.add_ifd_offset(Ifd::Primary, 4);

        self.decode_ifds()?;

        if self.makernote {
            self.makernote_register()?;
        }

        Ok(())
    }

    pub fn decode_header(&mut self) -> Result<()> {
        self.raw().seek_start(0_u32)?;

        self.read_byte_order()?;
        self.read_magic_42()?;

        let offset = self.raw().read_u32()?;
        self.raw().seek_start(offset)
    }

    pub fn read_byte_order(&mut self) -> Result<()> {
        let big_endian = match &self.raw().read_exact().e(Error::ByteOrderEof)? {
            b"II" => false,
            b"MM" => true,
            bo => return Err(Error::UnkownByteOrder(*bo)),
        };

        self.raw.big_endian = big_endian;

        Ok(())
    }

    pub fn read_magic_42(&mut self) -> Result<()> {
        match self.raw().read_u16().e(Error::MagicBytesEof)? {
            42 => Ok(()),
            magic => Err(Error::MagicBytesWrong(magic)),
        }
    }

    pub fn decode_ifds(&mut self) -> Result<()> {
        let ifd_offset = self.decode_entries(Ifd::Primary)?;
        if ifd_offset != 0 {
            self.raw().seek_start(ifd_offset)?;
            self.decode_entries(Ifd::Thumbnail)?;
        }

        Ok(())
    }

    pub fn decode_entries(&mut self, ifd: Ifd) -> Result<u32> {
        let n_entries: u16 = self.raw().read_u16().e(Error::NumerEntriesEof)?;
        tracing::debug!(
            "Reading {ifd:?} with {n_entries} entries at {}",
            self.raw().position()?
        );

        let mut exif_specific_ifd_offsets = Vec::new();
        for _ in 0..n_entries {
            let (tag, location) = self.read_entry()?;

            if let Some(ifd) = tag.exif_specific_ifd() {
                exif_specific_ifd_offsets.push((ifd, location));
            }

            self.locations
                .entry(TagIfd::new(tag, ifd))
                .or_default()
                .push(location);
        }

        let offset_location = self.raw().position()?;
        let ifd_offset = self.raw().read_u32()?;
        if ifd_offset > 0 && ifd == Ifd::Primary {
            self.add_ifd_offset(Ifd::Thumbnail, offset_location);
        }

        for (ifd, entry) in exif_specific_ifd_offsets {
            let offset = entry.value_offset.u32();
            self.add_ifd_offset(ifd, entry.value_offset_position());
            self.raw().seek_start(offset)?;
            self.decode_entries(ifd)?;
        }

        Ok(ifd_offset)
    }

    pub fn add_ifd_offset(&mut self, ifd: Ifd, location: u32) {
        let exists = self.ifd_locations.insert(ifd, location).is_some();

        if exists {
            tracing::warn!("Exif: Warning: Ifd exists twice {ifd:?}");
        }
    }
}
