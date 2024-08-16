use std::io::{Cursor, Read};

pub const MAGIC_BYTES: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];

pub struct Png<'a> {
    chunks: Vec<Chunk<'a>>,
}

pub struct Chunk<'a> {
    chunk_type: ChunkType,
    data: &'a [u8],
    crc: [u8; 4],
}

impl<'a> Chunk<'a> {
    pub fn chunk_type(&self) -> ChunkType {
        self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        self.data
    }

    pub fn crc(&self) -> &[u8] {
        &self.crc
    }
}

impl<'a> Png<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error> {
        let chunks = Self::find_chunks(data)?;

        Ok(Self { chunks })
    }

    pub fn chunks(&self) -> &[Chunk<'a>] {
        &self.chunks
    }

    fn find_chunks(data: &'a [u8]) -> Result<Vec<Chunk<'a>>, Error> {
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
            let chunk_data = data.get(data_start..data_end).ok_or(Error::UnexpectedEof)?;

            // Last 4 bytes after the data are a CRC
            cur.set_position(data_end as u64);
            let crc = &mut [0; 4];
            cur.read_exact(crc).map_err(|_| Error::UnexpectedEof)?;

            let chunk = Chunk {
                chunk_type,
                data: chunk_data,
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

#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    InvalidMagicBytes(Vec<u8>),
    PositionTooLarge,
}

pub const X: u32 = 1;

gufo_common::utils::convertible_enum!(
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    #[allow(non_camel_case_types)]
    pub enum ChunkType {
        /// Header
        IHDR = b(b"IHDR"),
        /// Image Data
        IDAT = b(b"IDAT"),
        /// End of file
        IEND = b(b"IEND"),

        /// Background Color
        bKGD = b(b"bKGD"),
        /// Embedded ICC profile
        iCCP = b(b"iCCP"),
        /// Physical pixel dimensions
        pHYs = b(b"pHYs"),
        /// Textual information
        tEXt = b(b"tEXt"),
        /// Image last-modification time
        tIME = b(b"tIME"),
    }
);

/// Convert bytes to u32
const fn b(d: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*d)
}
