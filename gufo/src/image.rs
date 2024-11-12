use gufo_common::error::ErrorWithData;

use crate::Error;

#[non_exhaustive]
#[derive(Debug)]
pub enum Image {
    #[cfg(feature = "png")]
    Png(gufo_png::Png),
    #[cfg(feature = "jpeg")]
    Jpeg(gufo_jpeg::Jpeg),
}

impl Image {
    #[cfg(any(feature = "jpeg", feature = "png", feature = "webp"))]
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        #[cfg(feature = "png")]
        if gufo_png::Png::is_filetype(&data) {
            let png = gufo_png::Png::new(data).map_err(|x| x.map_err(Error::Png))?;
            return Ok(Self::Png(png));
        }

        #[cfg(feature = "jpeg")]
        if gufo_jpeg::Jpeg::is_filetype(&data) {
            let jpeg = gufo_jpeg::Jpeg::new(data).map_err(|x| x.map_err(Error::Jpeg))?;
            return Ok(Self::Jpeg(jpeg));
        }

        Err(ErrorWithData::new(Error::NoSupportedFiletypeFound, data))
    }
}
