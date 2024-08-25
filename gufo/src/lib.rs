use gufo_common::error::ErrorWithData;
use gufo_common::geography;
use gufo_exif::Exif;
use gufo_xmp::Xmp;

const INFLATE_LIMIT: usize = 10_usize.pow(6) * 100;

#[derive(Debug, Default)]
pub struct RawMetadata {
    pub exif: Vec<Vec<u8>>,
    pub xmp: Vec<Vec<u8>>,
}

impl RawMetadata {
    pub fn for_guessed(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        if gufo_png::Png::is_filetype(&data) {
            let png = gufo_png::Png::new(data).map_err(|x| x.map_err(Error::Png))?;
            Ok(Self::for_png(&png))
        } else if gufo_jpeg::Jpeg::is_filetype(&data) {
            let jpeg = gufo_jpeg::Jpeg::new(data).map_err(|x| x.map_err(Error::Jpeg))?;
            Ok(Self::for_jpeg(&jpeg))
        } else if gufo_webp::WebP::is_filetype(&data) {
            let webp = gufo_webp::WebP::new(data).map_err(|x| x.map_err(Error::WebP))?;
            Ok(Self::for_webp(&webp))
        } else {
            Err(ErrorWithData::new(Error::NoSupportedFiletypeFound, data))
        }
    }

    pub fn for_png(png: &gufo_png::Png) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata
            .exif
            .extend(png.exif(INFLATE_LIMIT).map(|x| x.to_vec()));

        raw_metadata
    }

    pub fn for_jpeg(jpeg: &gufo_jpeg::Jpeg) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata
            .exif
            .extend(jpeg.exif_data().map(|x| x.to_vec()));

        raw_metadata.xmp.extend(jpeg.xmp_data().map(|x| x.to_vec()));

        raw_metadata
    }

    pub fn for_webp(jpeg: &gufo_webp::WebP) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata.exif.extend(jpeg.exif().map(|x| x.to_vec()));

        raw_metadata
    }

    pub fn into_metadata(self) -> Metadata {
        let mut metadata = Metadata::new();

        for exif in self.exif {
            let _ = metadata.add_raw_exif(exif);
        }

        for xmp in self.xmp {
            eprintln!("{}", String::from_utf8_lossy(&xmp));
            let _ = metadata.add_raw_xmp(xmp);
        }

        metadata
    }
}

#[derive(Debug, Default)]
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
    #[error("PNG: {0}")]
    Png(gufo_png::Error),
    #[error("JPEG: {0}")]
    Jpeg(gufo_jpeg::Error),
    #[error("Exif: {0}")]
    Exif(gufo_exif::error::Error),
    #[error("XMP: {0}")]
    Xmp(gufo_xmp::Error),
    #[error["WebP: {0}"]]
    WebP(gufo_webp::Error),
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_guessed(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        RawMetadata::for_guessed(data).map(|x| x.into_metadata())
    }

    pub fn for_png(png: &gufo_png::Png) -> Self {
        RawMetadata::for_png(png).into_metadata()
    }

    pub fn for_jpeg(jpeg: &gufo_jpeg::Jpeg) -> Self {
        RawMetadata::for_jpeg(jpeg).into_metadata()
    }

    pub fn add_raw_exif(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let mut exif = Exif::new(data).map_err(Error::Exif)?;
        let _ = exif.decoder().makernote_register();
        self.exif.push(exif);
        Ok(())
    }

    pub fn add_raw_xmp(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let xmp = Xmp::new(data).map_err(Error::Xmp)?;
        self.xmp.push(xmp);

        Ok(())
    }

    fn exif<T>(&self, exif_op: impl Fn(&Exif) -> Option<T>) -> Option<T> {
        self.exif.iter().find_map(exif_op)
    }

    fn xmp<T>(&self, xmp_op: impl Fn(&Xmp) -> Option<T>) -> Option<T> {
        self.xmp.iter().find_map(xmp_op)
    }

    fn exif_xmp<T>(
        &self,
        exif_op: impl Fn(&Exif) -> Option<T>,
        xmp_op: impl Fn(&Xmp) -> Option<T>,
    ) -> Option<T> {
        self.exif(exif_op).or_else(|| self.xmp(xmp_op))
    }

    pub fn gps_location(&self) -> Option<geography::Location> {
        self.exif(Exif::gps_location)
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.exif_xmp(Exif::date_time_original, Xmp::date_time_original)
    }

    pub fn f_number(&self) -> Option<f32> {
        self.exif_xmp(Exif::f_number, Xmp::f_number)
    }

    /// Exposure time in seconds
    pub fn exposure_time(&self) -> Option<(u32, u32)> {
        self.exif_xmp(Exif::exposure_time, Xmp::exposure_time)
    }

    /// Camera model
    pub fn model(&self) -> Option<String> {
        self.exif_xmp(Exif::model, Xmp::model)
    }

    /// Camera manifacturer
    pub fn make(&self) -> Option<String> {
        self.exif_xmp(Exif::make, Xmp::make)
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.exif_xmp(Exif::iso_speed_rating, Xmp::iso_speed_rating)
    }

    /// Focal length in millimeters
    pub fn focal_length(&self) -> Option<f32> {
        self.exif_xmp(Exif::focal_length, Xmp::focal_length)
    }

    pub fn camera_owner(&self) -> Option<String> {
        self.exif(Exif::camera_owner)
    }

    pub fn creator(&self) -> Option<String> {
        self.xmp(Xmp::creator)
    }
}
