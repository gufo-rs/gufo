use gufo_common::error::ErrorWithData;
use gufo_exif::Exif;
use gufo_xmp::Xmp;

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
        } else {
            Err(ErrorWithData::new(Error::NoSupportedFiletypeFound, data))
        }
    }

    pub fn for_png(png: &gufo_png::Png) -> Self {
        let mut raw_metadata = Self::default();

        raw_metadata
            .exif
            .extend(png.exif(10_usize.pow(6) * 100).map(|x| x.to_vec()));

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
        let exif = Exif::new(data).map_err(Error::Exif)?;
        self.exif.push(exif);
        Ok(())
    }

    pub fn add_raw_xmp(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let xmp = Xmp::new(data).map_err(Error::Xmp)?;
        self.xmp.push(xmp);

        Ok(())
    }

    pub fn model(&self) -> Option<String> {
        self.exif
            .iter()
            .find_map(|x| x.model())
            .or_else(|| self.xmp.iter().find_map(|x| x.model()))
    }

    pub fn creator(&self) -> Option<String> {
        self.xmp.iter().find_map(|x| x.creator())
    }
}
