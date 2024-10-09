pub use crate::*;

use std::io::{Cursor, Read, Seek};
use std::ops::Range;

pub const LEGACY_EXIF_KEYWORD: &[u8] = b"Raw profile type exif";

#[derive(Debug)]
pub struct Chunk<'a> {
    pub(crate) chunk_type: ChunkType,
    pub(crate) chunk_data_location: Range<usize>,
    pub(crate) crc: [u8; 4],
    pub(crate) png: &'a Png,
}

impl<'a> Chunk<'a> {
    pub fn chunk_type(&self) -> ChunkType {
        self.chunk_type
    }

    pub fn chunk_data(&self) -> &[u8] {
        self.png
            .data
            .get(self.chunk_data_location.clone())
            .expect("Unreachable: The chunk must be part of the data")
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

    pub fn unsafe_raw_chunk(self) -> RawChunk {
        RawChunk {
            chunk_type: self.chunk_type,
            chunk_data: self.chunk_data_location,
            crc: self.crc,
        }
    }
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

#[derive(Debug, Clone)]
pub struct RawChunk {
    pub(crate) chunk_type: ChunkType,
    pub(crate) chunk_data: Range<usize>,
    pub(crate) crc: [u8; 4],
}

impl RawChunk {
    pub(crate) fn chunk<'a>(&self, png: &'a Png) -> Chunk<'a> {
        Chunk {
            chunk_type: self.chunk_type,
            chunk_data_location: self.chunk_data.clone(),
            crc: self.crc,
            png,
        }
    }

    pub fn complete_data(&self) -> Range<usize> {
        (self
            .chunk_data
            .start
            .checked_sub(8)
            .expect("Unreachable: The chunk type and size must be part of the data"))
            ..(self
                .chunk_data
                .end
                .checked_add(4)
                .expect("Unreachable: The CBC musst be part of the data"))
    }

    pub fn total_len(&self) -> usize {
        self.complete_data().len()
    }
}
