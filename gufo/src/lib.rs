use gufo_exif::Exif;
use gufo_xmp::Xmp;

#[derive(Debug, Default)]
pub struct Metadata {
    exif: Vec<Exif>,
    xmp: Vec<Xmp>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_png(png: &gufo_png::Png) -> Self {
        let mut metadata = Self::new();

        if let Some(exif) = png.exif(10_usize.pow(6) * 100) {
            let _ = metadata.add_raw_exif(exif);
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
}

#[derive(Debug)]
pub enum Error {
    Exif(gufo_exif::error::Error),
    Xmp(gufo_xmp::Error),
}
