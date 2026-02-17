use std::ops::Range;

use gufo_common::read::ReadError;
use miniz_oxide::inflate::DecompressError;

use crate::ChunkType;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Read Error: {0}")]
    Read(#[from] ReadError),
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
    #[error("The chunk is not of type zTXt or tEXt")]
    NotTextualChunk,
    #[error("Expected chunk type {0:?} but got {1:?}")]
    UnexpectedChunkType(ChunkType, ChunkType),
    #[error("Unsupported pHYs unit: {0}")]
    UnsupportedPhysUnit(u8),
}
