#![doc = include_str!("../README.md")]

use std::io::{Cursor, Read, Seek};
use std::ops::Range;
use std::slice::SliceIndex;

use gufo_common::error::ErrorWithData;

pub const RIFF_MAGIC_BYTES: &[u8] = b"RIFF";
pub const WEBP_MAGIC_BYTES: &[u8] = b"WEBP";

#[derive(Debug, Clone)]
pub struct WebP {
    data: Vec<u8>,
    chunks: Vec<RawChunk>,
}

/// Representation of a WEBP image
impl WebP {
    /// Returns WEBP image representation
    ///
    /// * `data`: WEBP image data starting with RIFF magic byte
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        match Self::find_chunks(&data) {
            Ok(chunks) => Ok(Self { chunks, data }),
            Err(err) => Err(ErrorWithData::new(err, data)),
        }
    }

    pub fn is_filetype(data: &[u8]) -> bool {
        data.starts_with(RIFF_MAGIC_BYTES) && data.get(8..12) == Some(WEBP_MAGIC_BYTES)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn get(&self, index: impl SliceIndex<[u8], Output = [u8]>) -> Option<&[u8]> {
        self.data.get(index)
    }

    /// Returns all chunks
    pub fn chunks(&self) -> Vec<Chunk> {
        self.chunks.iter().map(|x| x.chunk(self)).collect()
    }

    pub fn exif(&self) -> Option<&[u8]> {
        self.chunks
            .iter()
            .find(|x| x.four_cc == FourCC::EXIF)
            .and_then(|x| self.get(x.payload.clone()))
    }

    /// List all chunks in the data
    fn find_chunks(data: &[u8]) -> Result<Vec<RawChunk>, Error> {
        let mut cur = Cursor::new(data);

        // Riff magic bytes
        let riff_magic_bytes = &mut [0; WEBP_MAGIC_BYTES.len()];
        cur.read_exact(riff_magic_bytes)
            .map_err(|_| Error::UnexpectedEof)?;
        if riff_magic_bytes != RIFF_MAGIC_BYTES {
            return Err(Error::RiffMagicBytesMissing(*riff_magic_bytes));
        }

        // File length
        let file_length_data = &mut [0; 4];
        cur.read_exact(file_length_data)
            .map_err(|_| Error::UnexpectedEof)?;
        let file_length = u32::from_le_bytes(*file_length_data);

        // Exif magic bytes
        let webp_magic_bytes = &mut [0; WEBP_MAGIC_BYTES.len()];
        cur.read_exact(webp_magic_bytes)
            .map_err(|_| Error::UnexpectedEof)?;
        if webp_magic_bytes != WEBP_MAGIC_BYTES {
            return Err(Error::WebpMagicBytesMissing(*webp_magic_bytes));
        }

        let mut chunks = Vec::new();
        loop {
            // Next 4 bytes are chunk FourCC (chunk type)
            let four_cc_data = &mut [0; 4];
            cur.read_exact(four_cc_data)
                .map_err(|_| Error::UnexpectedEof)?;
            let four_cc = FourCC::from(u32::from_le_bytes(*four_cc_data));

            // First 4 bytes are chunk size
            let size_data = &mut [0; 4];
            cur.read_exact(size_data)
                .map_err(|_| Error::UnexpectedEof)?;
            let size = u32::from_le_bytes(*size_data);

            // Next is the payload
            let payload_start: usize = cur
                .position()
                .try_into()
                .map_err(|_| Error::PositionTooLarge)?;
            let payload_end = payload_start
                .checked_add(size as usize)
                .ok_or(Error::PositionTooLarge)?;
            let payload = payload_start..payload_end;

            let chunk = RawChunk { four_cc, payload };

            // Jump to end of payload
            cur.set_position(payload_end as u64);

            // If odd, jump over 1 byte padding
            if size % 2 != 0 {
                cur.seek(std::io::SeekFrom::Current(1))
                    .map_err(|_| Error::UnexpectedEof)?;
            }

            chunks.push(chunk);

            if cur.position() >= file_length.into() {
                break;
            }
        }

        Ok(chunks)
    }
}

#[derive(Debug, Clone)]
pub struct RawChunk {
    four_cc: FourCC,
    payload: Range<usize>,
}

impl RawChunk {
    fn chunk<'a>(&self, webp: &'a WebP) -> Chunk<'a> {
        Chunk {
            four_cc: self.four_cc,
            payload: self.payload.clone(),
            webp,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk<'a> {
    four_cc: FourCC,
    payload: Range<usize>,
    webp: &'a WebP,
}

impl<'a> Chunk<'a> {
    pub fn four_cc(&self) -> FourCC {
        self.four_cc
    }

    pub fn payload(&self) -> &[u8] {
        self.webp
            .data
            .get(self.payload.clone())
            .expect("Unreachable: Chunk must be part of the data")
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("RIFF magic bytes missing: {0:?}")]
    RiffMagicBytesMissing([u8; 4]),
    #[error("WEBP magic bytes missing: {0:?}")]
    WebpMagicBytesMissing([u8; 4]),
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Position too large")]
    PositionTooLarge,
}

gufo_common::utils::convertible_enum!(
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    #[allow(non_camel_case_types)]
    /// Type of a chunk
    ///
    /// The value is stored as little endian [`u32`] of the original byte
    /// string.
    pub enum FourCC {
        /// Information about features used in the file
        VP8X = b(b"VP8X"),
        /// Embedded ICC color profile
        ICCP = b(b"ICCP"),
        /// Global parameters of the animation.
        ANIM = b(b"ANIM"),

        /// Information about a single frame
        ANMF = b(b"ANMF"),
        /// Alpha data for this frame (only with [`VP8`](Self::VP8))
        ALPH = b(b"ALPH"),
        /// Lossy data for this frame
        VP8 = b(b"VP8 "),
        /// Lossless data for this frame
        VP8L = b(b"VP8L"),

        EXIF = b(b"EXIF"),
        XMP = b(b"XMP "),
    }
);

impl FourCC {
    /// Returns the byte string of the chunk
    pub fn bytes(self) -> [u8; 4] {
        u32::to_le_bytes(self.into())
    }
}

/// Convert bytes to u32
const fn b(d: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*d)
}
