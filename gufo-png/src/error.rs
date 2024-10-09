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
}
