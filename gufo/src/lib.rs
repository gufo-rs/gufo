use gufo_common::error::ErrorWithData;
use gufo_exif::Exif;
use gufo_xmp::Xmp;

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
        let mut metadata = Self::new();

        if let Some(exif) = png.exif(10_usize.pow(6) * 100) {
            let _ = metadata.add_raw_exif(exif);
        }

        metadata
    }

    pub fn for_jpeg(jpeg: &gufo_jpeg::Jpeg) -> Self {
        let mut metadata = Self::new();

        for exif in jpeg.exif_data() {
            let _ = metadata.add_raw_exif(exif.to_vec());
        }

        for xmp in jpeg.xmp_data() {
            println!("{}", String::from_utf8_lossy(xmp));
            let _ = metadata.add_raw_xmp(xmp.to_vec());
        }

        metadata
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
