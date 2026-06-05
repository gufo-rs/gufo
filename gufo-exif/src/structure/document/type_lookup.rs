use gufo_common::exif::TagIfd;
use gufo_common::types::Rational;

use super::Document;
use crate::Error;
use crate::structure::util::Endieness;
use crate::structure::{Type, Typed};

impl<'a> Document<'a> {
    /// Lookup entry with arbitrary type
    pub fn lookup(&mut self, tag_ifd: TagIfd) -> Result<Option<Typed>, Error> {
        let endieness = self.endieness;

        let Some(entry) = self.entry_data(tag_ifd)? else {
            return Ok(None);
        };

        Typed::new(entry.type_, entry.count, entry.data, endieness).map(Some)
    }

    /// Lookup entry with ASCII or UTF-8 entry
    pub fn lookup_string_raw(&mut self, tag_ifd: TagIfd) -> Result<Option<String>, Error> {
        let Some(typed) = self.lookup(tag_ifd)? else {
            return Ok(None);
        };

        match typed {
            Typed::Ascii(ascii) => Ok(Some(String::from_utf8_lossy(&ascii).to_string())),
            Typed::Utf8(utf8) => Ok(Some(utf8)),
            _ => Err(Error::TypeMissmatch(
                typed.type_(),
                &[Type::Ascii, Type::Utf8],
            )),
        }
    }

    pub fn lookup_string(&mut self, tag_ifd: TagIfd) -> Result<Option<String>, Error> {
        let Some(mut s) = self.lookup_string_raw(tag_ifd)? else {
            return Ok(None);
        };

        if let Some(index) = s.find('\0') {
            s.truncate(index);
        }

        Ok(Some(s))
    }

    /// Lookup entry with multiple short values
    pub fn lookup_shorts(&mut self, tag_ifd: TagIfd) -> Result<Option<Vec<u16>>, Error> {
        let Some(typed) = self.lookup(tag_ifd)? else {
            return Ok(None);
        };

        if let Typed::Short(shorts) = typed {
            Ok(Some(shorts))
        } else {
            Err(Error::TypeMissmatch(typed.type_(), &[Type::Short]))
        }
    }

    /// Lookup entry with single short entry
    pub fn lookup_short(&mut self, tag_ifd: TagIfd) -> Result<Option<u16>, Error> {
        let Some(vec) = self.lookup_shorts(tag_ifd)? else {
            return Ok(None);
        };

        if let Some((first, rest)) = vec.split_first() {
            if !rest.is_empty() {
                Err(Error::ElementCountMissmatch(vec.len(), 1))
            } else {
                Ok(Some(*first))
            }
        } else {
            Ok(None)
        }
    }

    /// Lookup entry with multiple rational entries
    pub fn lookup_rationals<const N: usize>(
        &mut self,
        tag_ifd: TagIfd,
    ) -> Result<Option<[Rational<u32>; N]>, Error> {
        let Some(typed) = self.lookup(tag_ifd)? else {
            return Ok(None);
        };

        if let Typed::Rational(rationals) = typed {
            let len = rationals.len();
            Ok(Some(
                rationals
                    .try_into()
                    .map_err(|_| Error::ElementCountMissmatch(len, N))?,
            ))
        } else {
            Err(Error::TypeMissmatch(typed.type_(), &[Type::Rational]))
        }
    }

    /// Lookup entry with single rational entry
    pub fn lookup_rational(&mut self, tag_ifd: TagIfd) -> Result<Option<Rational<u32>>, Error> {
        Ok(self.lookup_rationals::<1>(tag_ifd)?.map(|x| x[0]))
    }

    /// Lookupe entry with character identified code
    ///
    /// Exif 3.0: 4.6.4. Character Identifier Code
    pub fn lookup_character_identified_code_string(
        &mut self,
        tagifd: TagIfd,
    ) -> Result<Option<String>, Error> {
        let Some(data) = self.lookup(tagifd)? else {
            return Ok(None);
        };

        // The standard only defines Undefined here, but others are used in the wild
        let data = match data {
            Typed::Undefined(unfedined) => unfedined,
            Typed::Ascii(ascii) => ascii,
            Typed::Utf8(utf8) => utf8.into_bytes(),
            _ => {
                return Err(Error::TypeMissmatch(data.type_(), &[Type::Undefined]));
            }
        };

        let s = if let Some(ascii) = data.strip_prefix(b"ASCII\0\0\0") {
            String::from_utf8_lossy(ascii).to_string()
        } else if let Some(unicode) = data.strip_prefix(b"UNICODE\0") {
            if let Ok(s) = String::from_utf8(unicode.to_vec()) {
                // First try UTF-8 as specified in Exif 3.0
                s
            } else {
                let u16_vec = match self.endieness {
                    Endieness::Big => unicode
                        .chunks_exact(2)
                        .map(|x| u16::from_be_bytes(x.try_into().unwrap()))
                        .collect::<Vec<_>>(),
                    Endieness::Litte => unicode
                        .chunks_exact(2)
                        .map(|x| u16::from_le_bytes(x.try_into().unwrap()))
                        .collect::<Vec<_>>(),
                };

                if let Ok(x) = String::from_utf16(&u16_vec) {
                    // Try UTF-16 otherwise, since Exif 2.3 can be interpreted as using UCS-2
                    x
                } else {
                    // Fallback to UTF-8 lossy if both don't work
                    String::from_utf8_lossy(unicode).to_string()
                }
            }
        } else {
            // Don't expect leading NULLs here since sometimes the content starts directly
            String::from_utf8_lossy(&data).to_string()
        };

        // Remove potential leading NULLs and all others since some cameras fill with
        // NULLs at the end
        let s = s.replace('\0', "");

        if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
    }
}
