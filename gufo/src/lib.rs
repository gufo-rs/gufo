mod image;

pub use gufo_common as common;
use gufo_common::error::ErrorWithData;
use gufo_common::geography;
use gufo_common::orientation::Orientation;
use gufo_common::prelude::*;
use gufo_exif::Exif;
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
}

impl RawMetadata {
    #[cfg(any(feature = "jpeg", feature = "png", feature = "webp"))]
    pub fn for_guessed(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        #[cfg(feature = "jpeg")]
        if gufo_jpeg::Jpeg::is_filetype(&data) {
            let jpeg = gufo_jpeg::Jpeg::new(data).map_err(|x| x.map_err(Error::Jpeg))?;
            return Ok(Self::for_jpeg(&jpeg));
        }

        #[cfg(feature = "png")]
        if gufo_png::Png::is_filetype(&data) {
            let png = gufo_png::Png::new(data).map_err(|x| x.map_err(Error::Png))?;
            return Ok(Self::for_png(&png));
        }

        #[cfg(feature = "tiff")]
        if gufo_tiff::Tiff::is_filetype(&data) {
            let tiff = gufo_tiff::Tiff::new(data).map_err(|x| x.map_err(Error::Tiff))?;
            return Ok(Self::for_tiff(&tiff));
        }

        #[cfg(feature = "webp")]
        if gufo_webp::WebP::is_filetype(&data) {
            let webp = gufo_webp::WebP::new(data).map_err(|x| x.map_err(Error::WebP))?;
            return Ok(Self::for_webp(&webp));
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
    exif: Vec<Exif>,
    xmp: Vec<Xmp>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic")]
    GenericError,
    #[error("NoSupportedFiletypeFound")]
    NoSupportedFiletypeFound,
    #[error("Exif: {0}")]
    Exif(gufo_exif::error::Error),
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
        RawMetadata::for_guessed(data).map(|x| x.into_metadata())
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
        let mut exif = Exif::new(data).map_err(Error::Exif)?;
        let _ = exif.decoder().makernote_register();
        self.exif.push(exif);
        Ok(())
    }

    pub fn exif(&self) -> &[Exif] {
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

    fn get_exif<T>(&self, exif_op: impl Fn(&Exif) -> Option<T>) -> Option<T> {
        self.exif.iter().find_map(exif_op)
    }

    fn get_xmp<T>(&self, xmp_op: impl Fn(&Xmp) -> Option<T>) -> Option<T> {
        self.xmp.iter().find_map(xmp_op)
    }

    fn exif_xmp<T>(
        &self,
        exif_op: impl Fn(&Exif) -> Option<T>,
        xmp_op: impl Fn(&Xmp) -> Option<T>,
    ) -> Option<T> {
        self.get_exif(exif_op).or_else(|| self.get_xmp(xmp_op))
    }

    pub fn camera_owner(&self) -> Option<String> {
        self.get_exif(Exif::camera_owner)
    }

    pub fn creator(&self) -> Option<String> {
        self.get_xmp(Xmp::creator)
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.exif_xmp(Exif::date_time_original, Xmp::date_time_original)
    }

    /// Exposure time in seconds
    pub fn exposure_time(&self) -> Option<(u32, u32)> {
        self.exif_xmp(Exif::exposure_time, Xmp::exposure_time)
    }

    pub fn f_number(&self) -> Option<f32> {
        self.exif_xmp(Exif::f_number, Xmp::f_number)
    }

    /// Focal length in millimeters
    pub fn focal_length(&self) -> Option<f32> {
        self.exif_xmp(Exif::focal_length, Xmp::focal_length)
    }

    pub fn gps_location(&self) -> Option<geography::Location> {
        self.get_exif(Exif::gps_location)
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.exif_xmp(Exif::iso_speed_rating, Xmp::iso_speed_rating)
    }

    /// Camera manifacturer
    pub fn make(&self) -> Option<String> {
        self.exif_xmp(Exif::make, Xmp::make)
    }

    /// Camera model
    pub fn model(&self) -> Option<String> {
        self.exif_xmp(Exif::model, Xmp::model)
    }

    pub fn orientation(&self) -> Option<Orientation> {
        // TODO: Should work from XMP as well
        self.get_exif(Exif::orientation)
    }

    pub fn software(&self) -> Option<String> {
        self.exif_xmp(Exif::software, Xmp::creator_tool)
    }

    pub fn user_comment(&self) -> Option<String> {
        self.get_exif(Exif::user_comment)
    }
}
