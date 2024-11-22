use crate::cicp::Cicp;

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
}

pub trait ImageComplete: ImageMetadata + ImageFormat {}
