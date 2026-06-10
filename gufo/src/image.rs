use gufo_common::cicp::Cicp;
use gufo_common::error::ErrorWithData;
use gufo_common::prelude::*;

use crate::Error;

#[non_exhaustive]
#[derive(Debug)]
pub enum Image {
    #[cfg(feature = "jpeg")]
    Jpeg(gufo_jpeg::Jpeg),
    #[cfg(feature = "png")]
    Png(gufo_png::Png),
    #[cfg(feature = "svg")]
    Svg(gufo_svg::Svg),
    #[cfg(feature = "tiff")]
    Tiff(gufo_tiff::Tiff),
    #[cfg(feature = "webp")]
    WebP(gufo_webp::WebP),
}

impl Image {
    #[cfg(any(feature = "jpeg", feature = "png", feature = "webp"))]
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        #[cfg(feature = "jpeg")]
        if gufo_jpeg::Jpeg::is_filetype(&data) {
            let jpeg = gufo_jpeg::Jpeg::new(data).map_err(|x| x.map_err(Error::Jpeg))?;
            return Ok(Self::Jpeg(jpeg));
        }

        #[cfg(feature = "png")]
        if gufo_png::Png::is_filetype(&data) {
            let png = gufo_png::Png::new(data).map_err(|x| x.map_err(Error::Png))?;
            return Ok(Self::Png(png));
        }

        #[cfg(feature = "tiff")]
        if gufo_tiff::Tiff::is_filetype(&data) {
            let tiff = gufo_tiff::Tiff::new(data).map_err(|x| x.map_err(Error::Tiff))?;
            return Ok(Self::Tiff(tiff));
        }

        #[cfg(feature = "webp")]
        if gufo_webp::WebP::is_filetype(&data) {
            let webp = gufo_webp::WebP::new(data).map_err(|x| x.map_err(Error::WebP))?;
            return Ok(Self::WebP(webp));
        }

        // Check SVG as last file type since the detection is most unstable and slow
        #[cfg(feature = "svg")]
        if gufo_svg::Svg::is_filetype(&data) {
            let svg = gufo_svg::Svg::new(data).map_err(|x| x.map_err(Error::Svg))?;
            return Ok(Self::Svg(svg));
        }

        Err(ErrorWithData::new(Error::NoSupportedFiletypeFound, data))
    }

    pub fn into_inner(self) -> Vec<u8> {
        match self {
            #[cfg(feature = "jpeg")]
            Self::Jpeg(jpeg) => jpeg.into_inner(),
            #[cfg(feature = "png")]
            Self::Png(png) => png.into_inner(),
            #[cfg(feature = "tiff")]
            Self::Tiff(tiff) => tiff.into_inner(),
            #[cfg(feature = "svg")]
            Self::Svg(svg) => svg.into_inner(),
            #[cfg(feature = "webp")]
            Self::WebP(webp) => webp.into_inner(),
        }
    }

    pub fn dyn_metadata(&self) -> Box<&dyn ImageMetadata> {
        match *self {
            #[cfg(feature = "jpeg")]
            Self::Jpeg(ref jpeg) => Box::new(jpeg as &dyn ImageMetadata),
            #[cfg(feature = "png")]
            Self::Png(ref png) => Box::new(png as &dyn ImageMetadata),
            #[cfg(feature = "svg")]
            Self::Svg(ref svg) => Box::new(svg as &dyn ImageMetadata),
            #[cfg(feature = "tiff")]
            Self::Tiff(ref tiff) => Box::new(tiff as &dyn ImageMetadata),
            #[cfg(feature = "webp")]
            Self::WebP(ref webp) => Box::new(webp as &dyn ImageMetadata),
        }
    }

    pub fn cicp(&self) -> Option<Cicp> {
        self.dyn_metadata().cicp()
    }
}
