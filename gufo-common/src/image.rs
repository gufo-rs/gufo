use std::collections::BTreeMap;

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
