mod high_level;
mod image;

use std::collections::BTreeMap;

pub use gufo_common as common;
use gufo_common::error::ErrorWithData;
use gufo_common::prelude::*;
use gufo_exif::ExifOwned;
#[cfg(feature = "jpeg")]
pub use gufo_jpeg as jpeg;
#[cfg(feature = "png")]
pub use gufo_png as png;
#[cfg(feature = "tiff")]
pub use gufo_tiff as tiff;
#[cfg(feature = "webp")]
pub use gufo_webp as webp;
use gufo_xmp::Xmp;
pub use image::Image;

#[derive(Debug, Default)]
pub struct RawMetadata {
    pub exif: Vec<Vec<u8>>,
    pub xmp: Vec<Vec<u8>>,
    pub key_value: BTreeMap<String, String>,
}

impl RawMetadata {
    #[cfg(any(feature = "jpeg", feature = "png", feature = "webp"))]
    pub fn for_guessed(data: Vec<u8>) -> Result<(Self, Vec<u8>), ErrorWithData<Error>> {
        #[cfg(feature = "jpeg")]
        if gufo_jpeg::Jpeg::is_filetype(&data) {
            let jpeg = gufo_jpeg::Jpeg::new(data).map_err(|x| x.map_err(Error::Jpeg))?;
            return Ok((Self::for_jpeg(&jpeg), jpeg.into_inner()));
        }

        #[cfg(feature = "png")]
        if gufo_png::Png::is_filetype(&data) {
            let png = gufo_png::Png::new(data).map_err(|x| x.map_err(Error::Png))?;
            return Ok((Self::for_png(&png), png.into_inner()));
        }

        #[cfg(feature = "tiff")]
        if gufo_tiff::Tiff::is_filetype(&data) {
            let tiff = gufo_tiff::Tiff::new(data).map_err(|x| x.map_err(Error::Tiff))?;
            return Ok((Self::for_tiff(&tiff), tiff.into_inner()));
        }

        #[cfg(feature = "webp")]
        if gufo_webp::WebP::is_filetype(&data) {
            let webp: gufo_webp::WebP =
                gufo_webp::WebP::new(data).map_err(|x| x.map_err(Error::WebP))?;
            return Ok((Self::for_webp(&webp), webp.into_inner()));
        }

        Err(ErrorWithData::new(Error::NoSupportedFiletypeFound, data))
    }

    #[cfg(feature = "jpeg")]
    pub fn for_jpeg(jpeg: &gufo_jpeg::Jpeg) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata
            .exif
            .extend(jpeg.exif_data().map(|x| x.to_vec()));

        raw_metadata.xmp.extend(jpeg.xmp_data().map(|x| x.to_vec()));

        raw_metadata
    }

    #[cfg(feature = "png")]
    pub fn for_png(png: &gufo_png::Png) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata.exif.extend(png.exif());
        raw_metadata.xmp.extend(png.xmp());
        raw_metadata.key_value.extend(png.key_value());

        raw_metadata
    }

    #[cfg(feature = "tiff")]
    pub fn for_tiff(tiff: &gufo_tiff::Tiff) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata.exif.extend(tiff.exif().map(|x| x.to_vec()));

        raw_metadata
    }

    #[cfg(feature = "webp")]
    pub fn for_webp(webp: &gufo_webp::WebP) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata.exif.extend(webp.exif().map(|x| x.to_vec()));

        raw_metadata
    }

    pub fn into_metadata(self) -> Metadata {
        let mut metadata = Metadata::new();

        for exif in self.exif {
            let _ = metadata.add_raw_exif(exif);
        }

        for xmp in self.xmp {
            let _ = metadata.add_raw_xmp(xmp);
        }

        metadata
    }
}

static_assertions::assert_impl_all!(Metadata: Send, Sync);

#[derive(Debug, Default, Clone)]
pub struct Metadata {
    exif: Vec<ExifOwned>,
    xmp: Vec<Xmp>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic")]
    GenericError,
    #[error("NoSupportedFiletypeFound")]
    NoSupportedFiletypeFound,
    #[error("Exif: {0}")]
    Exif(gufo_exif::Error),
    #[error("XMP: {0}")]
    Xmp(gufo_xmp::Error),

    #[cfg(feature = "jpeg")]
    #[error("JPEG: {0}")]
    Jpeg(gufo_jpeg::Error),
    #[cfg(feature = "png")]
    #[error("PNG: {0}")]
    Png(gufo_png::Error),
    #[cfg(feature = "tiff")]
    #[error["TIFF: {0}"]]
    Tiff(gufo_tiff::Error),
    #[cfg(feature = "webp")]
    #[error["WebP: {0}"]]
    WebP(gufo_webp::Error),
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(any(feature = "jpeg", feature = "png", feature = "tiff", feature = "webp"))]
    pub fn for_guessed(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        RawMetadata::for_guessed(data).map(|x| x.0.into_metadata())
    }

    #[cfg(feature = "jpeg")]
    pub fn for_jpeg(jpeg: &gufo_jpeg::Jpeg) -> Self {
        RawMetadata::for_jpeg(jpeg).into_metadata()
    }

    #[cfg(feature = "png")]
    pub fn for_png(png: &gufo_png::Png) -> Self {
        RawMetadata::for_png(png).into_metadata()
    }

    pub fn add_raw_exif(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let exif = ExifOwned::for_vec(data).map_err(Error::Exif)?;
        self.exif.push(exif);
        Ok(())
    }

    pub fn exif(&self) -> &[ExifOwned] {
        &self.exif
    }

    pub fn add_raw_xmp(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let xmp = Xmp::new(data).map_err(Error::Xmp)?;
        self.xmp.push(xmp);

        Ok(())
    }

    pub fn xmp(&self) -> &[Xmp] {
        &self.xmp
    }

    pub fn is_empty(&self) -> bool {
        self.xmp.is_empty() && self.exif.is_empty()
    }

    fn get_exif<T>(&self, exif_op: impl Fn(&ExifOwned) -> Option<T>) -> Option<T> {
        self.exif.iter().find_map(exif_op)
    }

    fn get_xmp<T>(&self, xmp_op: impl Fn(&Xmp) -> Option<T>) -> Option<T> {
        self.xmp.iter().find_map(xmp_op)
    }

    fn exif_xmp<T>(
        &self,
        exif_op: impl Fn(&ExifOwned) -> Option<T>,
        xmp_op: impl Fn(&Xmp) -> Option<T>,
    ) -> Option<T> {
        self.get_exif(exif_op).or_else(|| self.get_xmp(xmp_op))
    }
}
