use gufo_common::error::ErrorWithData;

const LE_MAGIC_BYTES: &[u8] = b"II*\0";
const BE_MAGIC_BYTES: &[u8] = b"MM\0*";

pub struct Tiff {
    data: Vec<u8>,
}

/// Representation of a TIFF image
impl Tiff {
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        Ok(Self { data })
    }

    pub fn is_filetype(data: &[u8]) -> bool {
        data.starts_with(LE_MAGIC_BYTES) || data.starts_with(BE_MAGIC_BYTES)
    }

    pub fn exif(&self) -> Result<&[u8], Error> {
        Ok(&self.data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
