use std::marker::PhantomData;
use std::sync::Mutex;

use gufo_common::exif::TagIfd;
use gufo_common::{geography, orientation};
use zerocopy::FromZeros;

use crate::structure::{Document, Rational};
use crate::Error;

#[derive(Debug)]
pub struct ExifInternal<'a, S: Storage<'a>> {
    document: S,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> Clone for ExifInternal<'a, OwnedStore> {
    fn clone(&self) -> Self {
        Self::for_vec(self.assemble().unwrap()).unwrap()
    }
}

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

pub struct BorrowedStore<'a> {
    document: Mutex<Document<'a>>,
}

impl<'a> Storage<'a> for BorrowedStore<'a> {
    fn access<T>(&self, f: impl FnOnce(&mut Document) -> T) -> T {
        f(&mut self.document.lock().unwrap())
    }
}

impl<'a> ExifInternal<'a, OwnedStore> {
    pub fn for_vec(data: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            document: OwnedStore::try_new(data, |x| {
                Ok::<_, Error>(Mutex::new(Document::for_mut_slice(x)?))
            })?,
            lifetime: Default::default(),
        })
    }
}

impl<'a> ExifInternal<'a, BorrowedStore<'a>> {
    pub fn for_mut_slice(data: &'a mut [u8]) -> Result<Self, Error> {
        let document = Document::for_mut_slice(data)?;
        Ok(Self {
            document: BorrowedStore {
                document: Mutex::new(document),
            },
            lifetime: Default::default(),
        })
    }
}

impl<'a> ExifInternal<'a, OwnedStore> {
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
        let mut raw = self.assemble()?;

        // Overwrite old list entry with remaining data
        raw.copy_within(pos_retain_begin..pos_retain_end, pos_retain_new);

        // Delete data that are now duplicated after the copy to not silently keep them
        // if the entry for them is deleted
        raw.get_mut(pos_obsolete_retain_start..pos_retain_end)
            .ok_or(Error::IndexOverflow)?
            .zero();

        // Parse exif again from raw data
        let exif = ExifInternal::for_vec(raw)?;

        // Replace internal parsed data with new data
        self.document = exif.document;

        Ok(true)
    }
}

impl<'a, S: Storage<'a>> ExifInternal<'a, S> {
    pub fn assemble(&self) -> Result<Vec<u8>, Error> {
        self.document(|x| x.assemble())
    }

    pub fn camera_owner_name(&self) -> Option<String> {
        self.document(|x| x.camera_owner_name())
    }

    pub fn document<T>(&self, f: impl FnOnce(&mut Document<'_>) -> T) -> T {
        self.document.access(|x| f(x))
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.document(|x| x.date_time_original())
    }

    /// Exposure time in seconds
    ///
    /// Fraction of first element devided by second element. The first element
    /// is typically one, such that the value is given in its common for like
    /// "1/60 sec".
    pub fn exposure_time(&self) -> Option<Rational<u32>> {
        self.document(|x| x.exposure_time())
    }

    /// Aperture
    pub fn f_number(&self) -> Option<f32> {
        self.document(|x| x.f_number())
    }

    /// Focal length in mm
    pub fn focal_length(&self) -> Option<f32> {
        self.document(|x| x.focal_length())
    }

    pub fn gps_location(&self) -> Option<geography::Location> {
        self.document(|x| x.gps_location())
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.document(|x| x.iso_speed_rating())
    }

    /// Camera manifacturer
    pub fn make(&self) -> Option<String> {
        self.document(|x| x.make())
    }

    /// Camera model
    pub fn model(&self) -> Option<String> {
        self.document(|x| x.model())
    }

    /// Image orientation
    ///
    /// Rotation and mirroring that have to be applied to show the image
    /// correctly
    pub fn orientation(&self) -> Option<orientation::Orientation> {
        self.document(|x| x.orientation())
    }

    pub fn software(&self) -> Option<String> {
        self.document(|x| x.software())
    }

    pub fn user_comment(&self) -> Option<String> {
        self.document(|x| x.user_comment())
    }
}
