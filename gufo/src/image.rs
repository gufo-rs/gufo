use gufo_common::error::ErrorWithData;

use crate::Error;

#[non_exhaustive]
#[derive(Debug)]
pub enum Image {
    Png(gufo_png::Png),
    Jpeg(gufo_jpeg::Jpeg),
}

impl Image {
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        if gufo_png::Png::is_filetype(&data) {
            let png = gufo_png::Png::new(data).map_err(|x| x.map_err(Error::Png))?;
            Ok(Self::Png(png))
        } else if gufo_jpeg::Jpeg::is_filetype(&data) {
            let jpeg = gufo_jpeg::Jpeg::new(data).map_err(|x| x.map_err(Error::Jpeg))?;
            Ok(Self::Jpeg(jpeg))
        } else {
            Err(ErrorWithData::new(Error::NoSupportedFiletypeFound, data))
        }
    }
}
