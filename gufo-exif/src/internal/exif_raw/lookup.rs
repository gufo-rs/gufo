use super::*;

impl super::ExifRaw {
    pub fn lookup_entry(&mut self, tagifd: impl Into<TagIfd>) -> Option<EntryRef> {
        self.locations
            .entry(tagifd.into())
            .or_default()
            .first()
            .copied()
    }

    pub fn lookup_data(&mut self, tagifd: impl Into<TagIfd>) -> Result<Option<(Type, Vec<u8>)>> {
        if let Some(entry) = self.lookup_entry(tagifd) {
            let value = match entry.value_offset {
                ValueOffset::Offset(offset) => {
                    self.raw().seek_start(offset)?;
                    let mut buf = vec![0; entry.data_len()?.usize()?];
                    self.raw
                        .buffer
                        .lock()
                        .unwrap()
                        .read_exact(&mut buf)
                        .e(Error::LookupEof)?;
                    buf
                }
                ValueOffset::Value(value) => u32::to_ne_bytes(value).to_vec(),
            };

            Ok(Some((entry.data_type, value)))
        } else {
            Ok(None)
        }
    }

    /// Read a single entry from IFD
    pub fn read_entry(&mut self) -> Result<(Tag, EntryRef)> {
        let position = self.raw().position()?;
        let tag_id = self.raw().read_u16().e(Error::EntryEof)?;
        let data_type = self.raw().read_u16().e(Error::EntryEof)?.into();
        let count = self.raw().read_u32().e(Error::EntryEof)?;
        let value = ValueOffset::new(data_type, count, self.raw().read_u32().e(Error::EntryEof)?)?;

        Ok((
            Tag(tag_id),
            EntryRef {
                position,
                data_type,
                count,
                value_offset: value,
            },
        ))
    }

    pub fn lookup_binary(&mut self, tagifd: impl Into<TagIfd>) -> Result<Option<Vec<u8>>> {
        Ok(self.lookup_data(tagifd.into())?.map(|(_, data)| data))
    }

    /// Returns a field of [`Type::Ascii`] or [`Type::Utf8`]
    ///
    /// This lookup deviates from the standard in that it removes all NULL bytes
    /// instead of just the last byte that should be NULL. This is due to many
    /// Exif implementations writing NULL bytes at the end or beginning of the
    /// data.
    pub fn lookup_string(&mut self, tagifd: impl Into<TagIfd>) -> Result<Option<String>> {
        let Some(data) = self.lookup_string_raw(tagifd)? else {
            return Ok(None);
        };

        // Strip all NULL bytes.
        let data = data.iter().cloned().filter(|x| *x != 0).collect::<Vec<_>>();

        Ok(Some(String::from_utf8_lossy(&data).to_string()))
    }

    /// Returns a field of [`Type::Ascii`] or [`Type::Utf8`]
    ///
    /// In contrast to [`Self::lookup_string`], this function leaves all NULL
    /// bytes in place.
    pub fn lookup_string_raw(&mut self, tagifd: impl Into<TagIfd>) -> Result<Option<Vec<u8>>> {
        let data = self.lookup_data(tagifd)?;
        if let Some((data_type, data)) = data {
            if data_type != Type::Ascii && data_type != Type::Utf8 {
                return Err(Error::WrongTypeGeneric);
            }

            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn lookup_short(&mut self, tagifd: impl Into<TagIfd>) -> Result<Option<u16>> {
        let Some(entry) = self.lookup_entry(tagifd.into()) else {
            return Ok(None);
        };

        Self::check_type(&entry, 1, Type::Short)?;

        if let ValueOffset::Value(x) = entry.value_offset {
            Ok(Some(if self.raw.big_endian {
                let bytes = x.to_be_bytes();
                u16::from_be_bytes([bytes[0], bytes[1]])
            } else {
                let bytes = x.to_le_bytes();
                u16::from_le_bytes([bytes[0], bytes[1]])
            }))
        } else {
            Err(Error::OffsetInsteadOfValue)
        }
    }

    pub fn lookup_long(&mut self, tagifd: TagIfd) -> Result<Option<u32>> {
        let Some(entry) = self.lookup_entry(tagifd) else {
            return Ok(None);
        };

        Self::check_type(&entry, 1, Type::Long)?;

        if let ValueOffset::Value(x) = entry.value_offset {
            Ok(Some(x))
        } else {
            Err(Error::OffsetInsteadOfValue)
        }
    }

    pub fn lookup_rational(&mut self, tagifd: TagIfd) -> Result<Option<(u32, u32)>> {
        let mut raw = self.raw();

        let Some(entry) = self.lookup_entry(tagifd) else {
            return Ok(None);
        };

        Self::check_type(&entry, 1, Type::Rational)?;

        let offset = entry.offset()?;

        raw.seek_start(offset)?;
        let x = raw.read_u32()?;
        let y = raw.read_u32()?;

        Ok(Some((x, y)))
    }

    pub fn lookup_rationals<const N: usize>(
        &mut self,
        tagifd: impl Into<TagIfd>,
    ) -> Result<Option<[(u32, u32); N]>> {
        let mut raw = self.raw();

        let Some(entry) = self.lookup_entry(tagifd) else {
            return Ok(None);
        };

        Self::check_type(&entry, N.u32()?, Type::Rational)?;

        let offset = entry.offset()?;
        raw.seek_start(offset)?;

        let mut rationals = [(0, 0); N];

        for rational in rationals.iter_mut() {
            let x = raw.read_u32()?;
            let y = raw.read_u32()?;

            *rational = (x, y);
        }

        Ok(Some(rationals))
    }

    pub fn lookup_rationals_f64<const N: usize>(
        &mut self,
        tagifd: impl Into<TagIfd>,
    ) -> Result<Option<[f64; N]>> {
        let Some(rationals): Option<[_; N]> = self.lookup_rationals(tagifd)? else {
            return Ok(None);
        };

        let mut floats = [0.; N];

        for (i, (x, y)) in rationals.into_iter().enumerate() {
            floats[i] = (x as f64).safe_div(y as f64)?;
        }

        Ok(Some(floats))
    }

    pub fn lookup_datetime(&mut self, tagifd: impl Into<TagIfd>) -> Result<Option<String>> {
        let Some(s) = self.lookup_string(tagifd)? else {
            return Ok(None);
        };

        Ok(Some(s.replacen(':', "-", 2).replacen(' ', "T", 1)))
    }

    fn check_type(entry: &EntryRef, count: u32, data_type: Type) -> Result<()> {
        if entry.count == count && entry.data_type == data_type {
            Ok(())
        } else {
            Err(Error::WrongType {
                expected: (count, data_type),
                actual: (entry.count, entry.data_type),
            })
        }
    }
}
