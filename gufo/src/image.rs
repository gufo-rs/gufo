use gufo_common::cicp::Cicp;
use gufo_common::error::ErrorWithData;
use gufo_common::prelude::*;

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

    pub fn into_inner(self) -> Vec<u8> {
        match self {
            #[cfg(feature = "jpeg")]
            Self::Jpeg(jpeg) => jpeg.into_inner(),
            #[cfg(feature = "png")]
            Self::Png(png) => png.into_inner(),
        }
    }

    pub fn dyn_metadata(&self) -> Box<&dyn ImageMetadata> {
        match *self {
            #[cfg(feature = "png")]
            Self::Png(ref png) => Box::new(png as &dyn ImageMetadata),
            #[cfg(feature = "jpeg")]
            Self::Jpeg(ref jpeg) => Box::new(jpeg as &dyn ImageMetadata),
        }
    }

    pub fn cicp(&self) -> Option<Cicp> {
        self.dyn_metadata().cicp()
    }
}
