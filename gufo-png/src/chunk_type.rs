use std::fmt::Debug;

gufo_common::utils::convertible_enum!(
    #[repr(u32)]
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        /// Primary chromaticities
        cHRM = b(b"cHRM"),
        /// Exif
        eXIf = b(b"eXIf"),
        /// Embedded ICC profile
        iCCP = b(b"iCCP"),
        /// Apple proprietary, information for faster image loading
        ///
        /// See <https://www.hackerfactor.com/blog/index.php?/archives/895-Connecting-the-iDOTs.html>
        iDOT = b(b"iDOT"),
        /// International textual data
        iTXt = b(b"iTXt"),
        /// Physical pixel dimensions
        pHYs = b(b"pHYs"),
        /// Image uses sRGB color space with the given rendering intent
        sRGB = b(b"sRGB"),
        /// Textual information
        tEXt = b(b"tEXt"),
        /// Image last-modification time
        tIME = b(b"tIME"),
        /// Compressed textual data
        zTXt = b(b"zTXt"),
    }
);

impl Debug for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.bytes();
        let name = String::from_utf8(bytes.to_vec())
            .ok()
            .and_then(|x| bytes.is_ascii().then_some(x))
            .unwrap_or_else(|| <Self as Into<u32>>::into(*self).to_string());

        match self {
            Self::Unknown(_) => write!(f, "Unknown({name:?})"),
            _ => f.write_str(&name),
        }
    }
}

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
