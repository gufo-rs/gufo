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
        let image = Image::new(data)?;

        Ok((Self::load(*image.dyn_metadata()), image.into_inner()))
    }

    fn load(metadata: &dyn ImageMetadata) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata.exif.extend(metadata.exif());
        raw_metadata.xmp.extend(metadata.xmp());
        raw_metadata.key_value.extend(metadata.key_value());

        raw_metadata
    }

    #[cfg(feature = "jpeg")]
    pub fn for_jpeg(jpeg: &gufo_jpeg::Jpeg) -> Self {
        Self::load(jpeg)
    }

    #[cfg(feature = "png")]
    pub fn for_png(png: &gufo_png::Png) -> Self {
        Self::load(png)
    }

    #[cfg(feature = "tiff")]
    pub fn for_tiff(tiff: &gufo_tiff::Tiff) -> Self {
        Self::load(tiff)
    }

    #[cfg(feature = "svg")]
    pub fn for_svg(svg: &gufo_svg::Svg) -> Self {
        Self::load(svg)
    }

    #[cfg(feature = "webp")]
    pub fn for_webp(webp: &gufo_webp::WebP) -> Self {
        Self::load(webp)
    }

    pub fn into_metadata(self) -> Metadata {
        let mut metadata = Metadata::new();

        for exif in self.exif {
            let _ = metadata.add_raw_exif(exif);
        }

        for xmp in self.xmp {
            let _ = metadata.add_raw_xmp(xmp);
        }

        metadata.key_value.extend(self.key_value);

        metadata
    }
}

static_assertions::assert_impl_all!(Metadata: Send, Sync);

#[derive(Debug, Default, Clone)]
pub struct Metadata {
    exif: Vec<ExifOwned>,
    xmp: Vec<Xmp>,
    key_value: BTreeMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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

    #[cfg(feature = "svg")]
    #[error["Svg: {0}"]]
    Svg(gufo_svg::Error),

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

    pub fn add_key_value(&mut self, data: BTreeMap<String, String>) -> Result<(), Error> {
        self.key_value.extend(data);

        Ok(())
    }

    pub fn xmp(&self) -> &[Xmp] {
        &self.xmp
    }

    pub fn is_empty(&self) -> bool {
        self.xmp.is_empty() && self.exif.is_empty()
    }

    fn lookup_exif<T>(&self, exif_op: impl Fn(&ExifOwned) -> Option<T>) -> Option<T> {
        self.exif.iter().find_map(exif_op)
    }

    fn lookup_keyval<T>(
        &self,
        keyval_op: impl Fn(&BTreeMap<String, String>) -> Option<T>,
    ) -> Option<T> {
        keyval_op(&self.key_value)
    }

    fn lookup_xmp<T>(&self, xmp_op: impl Fn(&Xmp) -> Option<T>) -> Option<T> {
        self.xmp.iter().find_map(xmp_op)
    }

    fn lookup_exif_xmp<T>(
        &self,
        exif_op: impl Fn(&ExifOwned) -> Option<T>,
        xmp_op: impl Fn(&Xmp) -> Option<T>,
    ) -> Option<T> {
        self.lookup_exif(exif_op)
            .or_else(|| self.lookup_xmp(xmp_op))
    }

    fn lookup_exif_xmp_keyval<T>(
        &self,
        exif_op: impl Fn(&ExifOwned) -> Option<T>,
        xmp_op: impl Fn(&Xmp) -> Option<T>,
        keyval_op: impl Fn(&BTreeMap<String, String>) -> Option<T>,
    ) -> Option<T> {
        self.lookup_exif(exif_op)
            .or_else(|| self.lookup_xmp(xmp_op))
            .or_else(|| self.lookup_keyval(keyval_op))
    }
}
