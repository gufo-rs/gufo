use gufo_common::field;

use crate::Error;

use super::Document;

impl<'a> Document<'a> {
    pub fn thumbnail_data(&mut self) -> Result<Option<&mut [u8]>, Error> {
        let data_start = self.lookup_long(field::ThumbnailJPEGInterchangeFormat.into())?;
        let data_len = self.lookup_long(field::ThumbnailJPEGInterchangeFormatLength.into())?;

        if let Some(data_start) = data_start
            && let Some(data_len) = data_len
        {
            self.data_start_len(data_start as usize, data_len as usize)
        } else {
            Ok(None)
        }
    }
}
