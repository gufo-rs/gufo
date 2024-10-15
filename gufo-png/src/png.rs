use std::io::{Cursor, Read};
use std::ops::Range;
use std::slice::SliceIndex;

pub use super::*;

pub const MAGIC_BYTES: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

#[derive(Debug, Clone)]
pub struct Png {
    /// Raw data
    pub(crate) data: Vec<u8>,
    /// Chunks in the order in which they appear in the data
    pub(crate) chunks: Vec<RawChunk>,
}

/// Representation of a PNG image
impl Png {
    /// Returns PNG image representation
    ///
    /// * `data`: PNG image data starting with magic byte
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        match Self::find_chunks(&data) {
            Ok(chunks) => Ok(Self { chunks, data }),
            Err(err) => Err(ErrorWithData::new(err, data)),
        }
    }

    /// Checks if passed data have PNG magic bytes
    pub fn is_filetype(data: &[u8]) -> bool {
        data.starts_with(MAGIC_BYTES)
    }

    /// Convert into raw data
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// Get part of the raw data
    pub fn get(&mut self, index: impl SliceIndex<[u8], Output = [u8]>) -> Option<&[u8]> {
        self.data.get(index)
    }

    /// Returns all chunks
    pub fn chunks(&self) -> Vec<Chunk> {
        self.chunks.iter().map(|x| x.chunk(self)).collect()
    }

    pub fn remove_chunk(&mut self, chunk: RawChunk) -> Result<(), Error> {
        self.data.drain(chunk.complete_data());
        self.chunks = Self::find_chunks(&self.data)?;
        Ok(())
    }

    /// Returns raw exif data if available
    ///
    /// Prefers the newer [`eXIf`](ChunkType::eXIf) chunk if available and uses
    /// the legacy [`zTXt`](ChunkType::zTXt) chunk with [`LEGACY_EXIF_KEYWORD`]
    /// as fallback.
    pub fn exif(&self, inflate_limit: usize) -> Option<Vec<u8>> {
        let chunks = self.chunks();

        if let Some(exif) = chunks.iter().find(|x| x.chunk_type() == ChunkType::eXIf) {
            Some(exif.chunk_data().to_vec())
        } else {
            chunks.iter().find_map(|x| x.legacy_exif(inflate_limit))
        }
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
            let chunk_start = cur
                .position()
                .try_into()
                .map_err(|_| Error::PositionTooLarge)?;
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

            let chunk_end = cur
                .position()
                .try_into()
                .map_err(|_| Error::PositionTooLarge)?;

            let chunk_complete = chunk_start..chunk_end;

            let chunk = RawChunk {
                chunk_type,
                chunk_data,
                chunk_complete,
                crc: *crc,
            };

            chunks.push(chunk);

            if chunk_type == ChunkType::IEND {
                break;
            }
        }

        Ok(chunks)
    }

    /// Replaces this PNG's image data with those from another
    ///
    /// Keeps all the metadata from this image but replaces the `IHDR` and
    /// `IDAT` chunks with the ones from `other`.
    pub fn replace_image_data(&mut self, other: &Self) -> Result<(), Error> {
        let Some(last_idat) = self
            .chunks
            .iter()
            .rev()
            .find(|x| x.chunk_type == ChunkType::IDAT)
        else {
            return Err(Error::NoIdatChunk);
        };

        let mut buf = Vec::with_capacity(other.data.len());
        buf.extend_from_slice(MAGIC_BYTES);

        for chunk in &self.chunks {
            match chunk.chunk_type {
                ChunkType::IHDR => {
                    let Some(new_header) = other
                        .chunks
                        .iter()
                        .find(|x| x.chunk_type == ChunkType::IHDR)
                    else {
                        return Err(Error::NoIhdrChunk);
                    };

                    buf.extend_from_slice(other.get_result(new_header.chunk_complete.clone())?);
                }
                ChunkType::iDOT => {
                    // Drop apples proprietary iDOT chunk since it depends on
                    // the IDAT data and IHDR and we neither know how to rewrite
                    // nor do we care.
                }
                ChunkType::IDAT => {
                    if chunk.chunk_data == last_idat.chunk_data {
                        for idat_chunk in other
                            .chunks
                            .iter()
                            .filter(|x| x.chunk_type == ChunkType::IDAT)
                        {
                            buf.extend_from_slice(
                                other.get_result(idat_chunk.chunk_complete.clone())?,
                            );
                        }
                    }
                }
                _ => {
                    buf.extend_from_slice(self.get_result(chunk.chunk_complete.clone())?);
                }
            }
        }

        self.chunks = dbg!(Self::find_chunks(&buf))?;
        self.data = buf;

        Ok(())
    }

    fn get_result(&self, index: Range<usize>) -> Result<&[u8], Error> {
        self.data
            .get(index.clone())
            .ok_or(Error::IndexNotInData(index))
    }
}
