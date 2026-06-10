use gufo_common::error::ErrorWithData;
use gufo_common::image::ImageMetadata;

const LE_MAGIC_BYTES: &[u8] = b"II*\0";
const BE_MAGIC_BYTES: &[u8] = b"MM\0*";

#[derive(Debug)]
pub struct Tiff {
    data: Vec<u8>,
}

impl ImageMetadata for Tiff {
    fn exif(&self) -> Vec<Vec<u8>> {
        vec![self.data.clone()]
    }
}

/// Representation of a TIFF image
impl Tiff {
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        Ok(Self { data })
    }

    pub fn is_filetype(data: &[u8]) -> bool {
        data.starts_with(LE_MAGIC_BYTES) || data.starts_with(BE_MAGIC_BYTES)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
