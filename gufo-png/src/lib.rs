use std::{
    io::{Cursor, Read, Seek},
    ops::Range,
};

use miniz_oxide::inflate::DecompressError;

pub const MAGIC_BYTES: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

pub const LEGACY_EXIF_KEYWORD: &[u8] = b"Raw profile type exif";

#[derive(Debug, Clone)]
pub struct Png {
    data: Vec<u8>,
    chunks: Vec<RawChunk>,
}

/// Representation of a PNG image
impl Png {
    /// Returns PNG image representation
    ///
    /// * `data`: PNG image data starting with magic byte
    pub fn new(data: Vec<u8>) -> Result<Self, Error> {
        let chunks = Self::find_chunks(&data)?;

        Ok(Self { chunks, data })
    }

    /// Returns all chunks
    pub fn chunks(&self) -> Vec<Chunk> {
        self.chunks.iter().map(|x| x.chunk(self)).collect()
    }

    pub fn chunks_with_position(&self) -> Vec<(usize, Chunk)> {
        let mut pos = MAGIC_BYTES.len();

        let mut chunks = Vec::new();
        for chunk in &self.chunks {
            chunks.push((pos, chunk.chunk(self)));
            pos = pos.checked_add(chunk.total_len()).unwrap();
        }

        chunks
    }

    /// List all chunks in the data
    fn find_chunks(data: &[u8]) -> Result<Vec<RawChunk>, Error> {
        let mut cur = Cursor::new(data);
        let magic_bytes = &mut [0; MAGIC_BYTES.len()];

        cur.read_exact(magic_bytes)
            .map_err(|_| Error::UnexpectedEof)?;

        if magic_bytes != MAGIC_BYTES {
            return Err(Error::InvalidMagicBytes(magic_bytes.to_vec()));
        }

        let mut chunks = Vec::new();
        loop {
            // First 4 bytes are length
            let length_data = &mut [0; 4];
            cur.read_exact(length_data)
                .map_err(|_| Error::UnexpectedEof)?;
            let length = u32::from_be_bytes(*length_data);

            // Next 4 bytes are chunk type
            let chunk_type_data = &mut [0; 4];
            cur.read_exact(chunk_type_data)
                .map_err(|_| Error::UnexpectedEof)?;
            let chunk_type = ChunkType::from(u32::from_be_bytes(*chunk_type_data));

            // Next are the data
            let data_start: usize = cur
                .position()
                .try_into()
                .map_err(|_| Error::PositionTooLarge)?;
            let data_end = data_start
                .checked_add(length as usize)
                .ok_or(Error::PositionTooLarge)?;
            let chunk_data = data_start..data_end;

            // Last 4 bytes after the data are a CRC
            cur.set_position(data_end as u64);
            let crc = &mut [0; 4];
            cur.read_exact(crc).map_err(|_| Error::UnexpectedEof)?;

            let chunk = RawChunk {
                chunk_type,
                chunk_data,
                crc: *crc,
            };

            chunks.push(chunk);

            if chunk_type == ChunkType::IEND {
                break;
            }
        }

        Ok(chunks)
    }
}

#[derive(Debug, Clone)]
pub struct RawChunk {
    chunk_type: ChunkType,
    chunk_data: Range<usize>,
    crc: [u8; 4],
}

impl RawChunk {
    fn chunk<'a>(&self, png: &'a Png) -> Chunk<'a> {
        Chunk {
            chunk_type: self.chunk_type,
            chunk_data_location: self.chunk_data.clone(),
            crc: self.crc,
            png,
        }
    }

    pub fn total_len(&self) -> usize {
        self.chunk_data.len().checked_add(8).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Chunk<'a> {
    chunk_type: ChunkType,
    chunk_data_location: Range<usize>,
    crc: [u8; 4],
    png: &'a Png,
}

impl<'a> Chunk<'a> {
    pub fn total_len(&self) -> usize {
        self.chunk_data_location.len().checked_add(8).unwrap()
    }

    pub fn chunk_type(&self) -> ChunkType {
        self.chunk_type
    }

    pub fn chunk_data(&self) -> &[u8] {
        self.png.data.get(self.chunk_data_location.clone()).unwrap()
    }

    pub fn crc(&self) -> &[u8] {
        &self.crc
    }

    pub fn keyword(&self) -> Result<&[u8], Error> {
        let data = self.chunk_data();

        let keyword_length = data.iter().take_while(|x| **x != 0).count();

        data.get(..keyword_length)
            .ok_or(Error::UnexpectedEndOfChunkData)
    }

    /// Returns keyword and value of a [`tEXt`](ChunkType::tEXt) chunk
    pub fn text(&self) -> Result<(&[u8], &[u8]), Error> {
        let keyword = self.keyword()?;
        let data_start = keyword
            .len()
            .checked_add(1)
            .ok_or(Error::PositionTooLarge)?;

        let text = self
            .chunk_data()
            .get(data_start..)
            .ok_or(Error::UnexpectedEndOfChunkData)?;

        Ok((keyword, text))
    }

    /// Returns the content of a [`zTXt`](ChunkType::zTXt) chunk
    ///
    /// The first value is the keyword, the second is the decompressed data.
    pub fn ztxt(&self, inflate_limit: usize) -> Result<(&[u8], Vec<u8>), Error> {
        let (keyword, raw) = self.text()?;

        // Skip byte for compression type
        let raw = raw.get(1..).ok_or(Error::UnexpectedEndOfChunkData)?;

        let data = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(raw, inflate_limit)
            .map_err(Error::Zlib)?;

        Ok((keyword, data))
    }

    /// Returns the Exif data stored in a [`zTXt`](ChunkType::zTXt) chunk
    pub fn legacy_exif(&self, inflate_limit: usize) -> Option<Vec<u8>> {
        if self.keyword().ok()? != LEGACY_EXIF_KEYWORD {
            return None;
        }

        let (_, raw) = self.ztxt(inflate_limit).ok()?;
        let mut cur = Cursor::new(&raw);

        // Skip whitespaces
        skip_while(&mut cur, |x| x.is_ascii_whitespace()).ok()?;

        let exif = &mut [0; 4];
        cur.read_exact(exif).ok()?;
        if exif != b"exif" {
            return None;
        }

        // Skip whitespaces
        skip_while(&mut cur, |x| x.is_ascii_whitespace()).ok()?;

        // Skip numbers (data length)
        skip_while(&mut cur, |x| x.is_ascii_digit()).ok()?;

        // Skip whitespaces
        skip_while(&mut cur, |x| x.is_ascii_whitespace()).ok()?;

        // Data without whitespaces
        let data = raw
            .iter()
            .skip(cur.position().try_into().ok()?)
            .filter(|c| !c.is_ascii_whitespace())
            .cloned()
            .collect::<Vec<u8>>();

        // Decode data from hex
        let exif_with_prefix = hex::decode(data).ok()?;

        // Drop header
        exif_with_prefix
            .strip_prefix(b"Exif\0\0")
            .map(|x| x.to_vec())
    }

    pub fn raw_chunk(self) -> RawChunk {
        RawChunk {
            chunk_type: self.chunk_type,
            chunk_data: self.chunk_data_location,
            crc: self.crc,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    InvalidMagicBytes(Vec<u8>),
    PositionTooLarge,
    UnexpectedEndOfChunkData,
    Zlib(DecompressError),
}

gufo_common::utils::convertible_enum!(
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    #[allow(non_camel_case_types)]
    /// Type of a chunk
    ///
    /// The value is stored as big endian [`u32`] of the original byte string.
    pub enum ChunkType {
        /// Header
        IHDR = b(b"IHDR"),
        /// Image Data
        IDAT = b(b"IDAT"),
        /// End of file
        IEND = b(b"IEND"),

        /// Background Color
        bKGD = b(b"bKGD"),
        /// Exif
        eXIf = b(b"eXIf"),
        /// Embedded ICC profile
        iCCP = b(b"iCCP"),
        /// International textual data
        iTXt = b(b"iTXt"),
        /// Physical pixel dimensions
        pHYs = b(b"pHYs"),
        /// Textual information
        tEXt = b(b"tEXt"),
        /// Image last-modification time
        tIME = b(b"tIME"),
        /// Compressed textual data
        zTXt = b(b"zTXt"),
    }
);

impl ChunkType {
    /// Returns the byte string of the chunk
    pub fn bytes(self) -> [u8; 4] {
        u32::to_be_bytes(self.into())
    }
}

/// Convert bytes to u32
const fn b(d: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*d)
}

// Moves cursor in front of the first element that does not fulfill the
// predicate
fn skip_while(
    cur: &mut Cursor<&Vec<u8>>,
    predicate: impl Fn(u8) -> bool,
) -> Result<(), std::io::Error> {
    let c: &mut [u8; 1] = &mut [0];

    loop {
        cur.read_exact(c)?;
        if !predicate(c[0]) {
            break;
        }
    }

    cur.seek(std::io::SeekFrom::Current(-1))?;

    Ok(())
}
