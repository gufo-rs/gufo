use std::ops::Range;

use miniz_oxide::inflate::DecompressError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Invalid magic bytes: {0:x?}")]
    InvalidMagicBytes(Vec<u8>),
    #[error("Position too large")]
    PositionTooLarge,
    #[error("Unexpected end of chunk data")]
    UnexpectedEndOfChunkData,
    #[error("Zlib decompression error: {0}")]
    Zlib(DecompressError),
    #[error("Data don't contain a single IDAT (image data) chunk.")]
    NoIdatChunk,
    #[error("Data don't contain a IHDR (header) chunk.")]
    NoIhdrChunk,
    #[error("The requested range '{0:?}' is not part of the image data")]
    IndexNotInData(Range<usize>),
}
