#![doc = include_str!("../README.md")]

mod chunk;
mod chunk_type;
mod error;
mod png;

pub use chunk::*;
pub use chunk_type::*;
pub use error::*;
use gufo_common::error::ErrorWithData;
pub use png::*;

/// Remove chunk from PNG
///
/// ```
/// let data = std::fs::read("../test-images/images/exif/exif.png").unwrap();
/// let mut png = gufo_png::Png::new(data).unwrap();
///
/// assert_eq!(png.chunks().len(), 43);
///
/// // Find one Exif chunk
/// let chunk = png
///     .chunks()
///     .into_iter()
///     .find(|x| x.chunk_type() == gufo_png::ChunkType::eXIf)
///     .unwrap();
///
/// // Remove that Exif chunk
/// gufo_png::remove_chunk!(png, chunk);
///
/// assert_eq!(png.chunks().len(), 42);
/// ```
#[macro_export]
macro_rules! remove_chunk {
    ($png:ident, $chunk:expr) => {{
        let raw_chunk = $chunk.unsafe_raw_chunk();
        $png.remove_chunk(raw_chunk)
    }};
}
