use std::borrow::Cow;
use std::io::{Cursor, Read, Seek};
use std::ops::Range;

use gufo_common::read::{ReadExt, SliceExt};

pub use crate::*;

pub const LEGACY_EXIF_KEYWORD: &[u8] = b"Raw profile type exif";
pub const LEGACY_XMP_KEYWORD: &[u8] = b"Raw profile type xmp";
pub const XMP_KEYWORD: &[u8] = b"XML:com.adobe.xmp";

#[derive(Debug)]
pub struct Chunk<'a> {
    pub(crate) raw_chunk: RawChunk,
    pub(crate) png: &'a Png,
}

impl<'a> Chunk<'a> {
    pub fn chunk_type(&self) -> ChunkType {
        self.raw_chunk.chunk_type
    }

    pub fn chunk_data(&self) -> &'a [u8] {
        self.png
            .data
            .get(self.raw_chunk.chunk_data.clone())
            .expect("Unreachable: The chunk must be part of the data")
    }

    pub fn complete_data(&self) -> &[u8] {
        self.png
            .data
            .get(self.raw_chunk.chunk_complete.clone())
            .expect("Unreachable: The chunk must be part of the data")
    }

    pub fn crc(&self) -> &[u8] {
        &self.raw_chunk.crc
    }

    pub fn keyword(&self) -> Result<&[u8], Error> {
        let data = self.chunk_data();

        let keyword_length = data.iter().take_while(|x| **x != 0).count();

        data.get(..keyword_length)
            .ok_or(Error::UnexpectedEndOfChunkData)
    }

    /// Returns the contents of an [`iTXt`](ChunkType::iTXt) chunk
    pub fn itxt(&self) -> Result<Itxt<'_>, Error> {
        let mut cur = Cursor::new(self.chunk_data());

        let keyword = cur.slice_until(b'\0')?;
        let compression = cur.read_byte()?;
        let _compression_method = cur.read_byte()?;
        let language = cur.slice_until(b'\0')?;
        let translated_keyword = String::from_utf8_lossy(cur.slice_until(b'\0')?);
        let raw_text = cur.slice_to_end()?;

        let text = if compression == 1 {
            Cow::Owned(
                String::from_utf8_lossy(
                    &miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(raw_text, 10000000)
                        .map_err(Error::Zlib)?,
                )
                .to_string(),
            )
        } else {
            String::from_utf8_lossy(raw_text)
        };

        Ok(Itxt {
            keyword,
            language,
            translated_keyword,
            text,
        })
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

    /// Returns the content of a [`tEXt`](ChunkType::tEXt) or
    /// [`zTXt`](ChunkType::zTXt) chunk
    ///
    /// The first value is the keyword, the second is the decompressed data.
    pub fn textual(&self, inflate_limit: usize) -> Result<(&[u8], Vec<u8>), Error> {
        match self.chunk_type() {
            ChunkType::tEXt => self.text().map(|(k, v)| (k, v.to_vec())),
            ChunkType::zTXt => self.ztxt(inflate_limit),
            _ => Err(Error::NotTextualChunk),
        }
    }

    /// XMP data
    ///
    /// XMP data stored in accordance with XMP Specification Part 3: Storage in
    /// Files, Section 1.1.5
    pub fn xmp(&self) -> Result<Option<Cow<'_, str>>, Error> {
        if self.chunk_type() == ChunkType::iTXt && self.keyword()? == XMP_KEYWORD {
            return Ok(Some(self.itxt()?.text));
        }

        Ok(None)
    }

    /// Returns the XMP data stored in a [`tEXt`](ChunkType::tEXt) or
    /// [`zTXt`](ChunkType::zTXt) chunk
    pub fn legacy_xmp(&self, inflate_limit: usize) -> Option<Vec<u8>> {
        if self.keyword().ok()? != LEGACY_XMP_KEYWORD {
            return None;
        }

        let (_, raw) = self.textual(inflate_limit).ok()?;
        let mut cur = Cursor::new(&raw);

        // Skip whitespaces
        skip_while(&mut cur, |x| x.is_ascii_whitespace()).ok()?;

        let xmp = &mut [0; 3];
        cur.read_exact(xmp).ok()?;
        if xmp != b"xmp" {
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
        hex::decode(data).ok()
    }

    /// Returns the Exif data stored in a [`tEXt`](ChunkType::tEXt) or
    /// [`zTXt`](ChunkType::zTXt) chunk
    pub fn legacy_exif(&self, inflate_limit: usize) -> Option<Vec<u8>> {
        if self.keyword().ok()? != LEGACY_EXIF_KEYWORD {
            return None;
        }

        let (_, raw) = self.textual(inflate_limit).ok()?;
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
        self.raw_chunk
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
    pub(crate) chunk_complete: Range<usize>,
    pub(crate) crc: [u8; 4],
}

impl RawChunk {
    pub(crate) fn chunk<'a>(&self, png: &'a Png) -> Chunk<'a> {
        Chunk {
            raw_chunk: self.clone(),
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

pub struct Itxt<'a> {
    pub keyword: &'a [u8],
    pub language: &'a [u8],
    pub translated_keyword: Cow<'a, str>,
    pub text: Cow<'a, str>,
}

pub struct NewChunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl NewChunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> NewChunk {
        NewChunk { chunk_type, data }
    }

    /// Create new tEXt chunk
    pub fn text(keyword: &str, text: &str) -> NewChunk {
        let mut text_encoded = vec![0; text.len()];
        let len =
            encoding_rs::mem::convert_utf8_to_latin1_lossy(text.as_bytes(), &mut text_encoded);
        text_encoded.truncate(len);

        let mut data = keyword.as_bytes().to_vec();
        data.push(0);
        data.extend(text_encoded);

        NewChunk {
            chunk_type: ChunkType::tEXt,
            data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.data.len());

        buf.extend((self.data.len() as u32).to_be_bytes());
        buf.extend(self.chunk_type.bytes());
        buf.extend(&self.data);

        let crc = crc32fast::hash(&buf[4..]);

        buf.extend(crc.to_be_bytes());

        buf
    }
}
