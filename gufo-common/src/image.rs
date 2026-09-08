use std::collections::BTreeMap;
use std::fmt::Display;

use crate::cicp::Cicp;
use crate::physical_dimension::PixelDensity;

pub trait ImageFormat {
    /// Usually checks if data start with correct magic bytes
    fn is_filetype(data: &[u8]) -> bool;
}

pub trait ImageMetadata {
    fn cicp(&self) -> Option<Cicp> {
        None
    }

    fn exif(&self) -> Vec<Vec<u8>> {
        Vec::new()
    }

    fn set_exif(&mut self, _exif_data: &[u8]) -> Result<(), ImageMetadataError> {
        Err(ImageMetadataError::OperationUnsupported)
    }

    fn xmp(&self) -> Vec<Vec<u8>> {
        Vec::new()
    }

    fn key_value(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn pixel_density(&self) -> Option<PixelDensity> {
        None
    }
}

pub trait ImageComplete: ImageMetadata + ImageFormat {}

#[derive(Debug, Clone)]
pub enum ImageMetadataError {
    OperationUnsupported,
    Other(String),
}

impl std::error::Error for ImageMetadataError {}

impl std::fmt::Display for ImageMetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OperationUnsupported => f.write_str("Image metadata operation not support"),
            Self::Other(msg) => write!(f, "Image metadata error: {msg}"),
        }
    }
}

impl ImageMetadataError {
    pub fn other(msg: impl Display) -> Self {
        Self::Other(msg.to_string())
    }
}
