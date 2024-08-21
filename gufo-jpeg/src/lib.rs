use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::Range;

use gufo_common::error::ErrorWithData;

pub const EXIF_IDENTIFIER_STRING: &[u8] = b"Exif\0\0";
pub const XMP_IDENTIFIER_STRING: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

pub const MAGIC_BYTES: &[u8] = &[0xFF, 0xD8, 0xFF];

pub const MARKER_START: u8 = 0xFF;

#[derive(Debug)]
pub struct Jpeg {
    segments: Vec<RawSegment>,
    data: Vec<u8>,
}

impl Jpeg {
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        match Self::find_segments(&data) {
            Ok(segments) => Ok(Self { segments, data }),
            Err(err) => Err(ErrorWithData::new(err, data)),
        }
    }

    pub fn is_filetype(data: &[u8]) -> bool {
        data.starts_with(MAGIC_BYTES)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// List all segments in their order of appearance
    pub fn segments(&self) -> Vec<Segment> {
        self.segments.iter().map(|x| x.segment(self)).collect()
    }

    /// List all segments with the given marker
    pub fn segments_marker(&self, marker: Marker) -> impl Iterator<Item = Segment> {
        self.segments
            .iter()
            .filter(move |x| x.marker == marker)
            .map(|x| x.segment(self))
    }

    pub fn exif(&self) -> impl Iterator<Item = Segment> {
        self.segments_marker(Marker::APP1)
            .filter(|x| x.data().starts_with(EXIF_IDENTIFIER_STRING))
    }

    pub fn exif_data(&self) -> impl Iterator<Item = &[u8]> {
        self.exif()
            .filter_map(|x| x.data().get(EXIF_IDENTIFIER_STRING.len()..))
    }

    pub fn xmp(&self) -> impl Iterator<Item = Segment> {
        self.segments_marker(Marker::APP1)
            .filter(|x| x.data().starts_with(XMP_IDENTIFIER_STRING))
    }

    pub fn xmp_data(&self) -> impl Iterator<Item = &[u8]> {
        self.xmp()
            .filter_map(|x| x.data().get(XMP_IDENTIFIER_STRING.len()..))
    }

    fn find_segments(data: &[u8]) -> Result<Vec<RawSegment>, Error> {
        let mut source = Cursor::new(data);

        let buf = &mut [0; 2];
        source.read_exact(buf).map_err(|_| Error::UnexpectedEof)?;

        if data.get(..MAGIC_BYTES.len()) != Some(MAGIC_BYTES) {
            return Err(Error::InvalidMagicBytes(*buf));
        }

        let mut segments = Vec::new();
        loop {
            // Read tag
            source.read_exact(buf).map_err(|_| Error::UnexpectedEof)?;
            tracing::debug!("Found tag {buf:x?}");

            if buf[0] != MARKER_START {
                return Err(Error::ExpectedMarkerStart(buf[0]));
            }

            let marker = Marker::from(buf[1]);
            let pos = source.stream_position().unwrap();

            // Read len
            source.read_exact(buf).map_err(|_| Error::UnexpectedEof)?;
            let len: u16 = u16::from_be_bytes(*buf);

            let segment = RawSegment {
                marker,
                data: (pos + 2) as usize..(pos as usize + len as usize),
            };

            segments.push(segment);

            if marker == Marker::SOS {
                break;
            }
            source.seek(SeekFrom::Current(len as i64 - 2)).unwrap();
        }

        Ok(segments)
    }
}

#[derive(Debug)]
pub struct RawSegment {
    marker: Marker,
    data: Range<usize>,
}

impl RawSegment {
    pub fn segment<'a>(&self, jpeg: &'a Jpeg) -> Segment<'a> {
        Segment {
            marker: self.marker,
            data: self.data.clone(),
            jpeg,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Segment<'a> {
    marker: Marker,
    data: Range<usize>,
    jpeg: &'a Jpeg,
}

impl<'a> Segment<'a> {
    pub fn marker(&self) -> Marker {
        self.marker
    }

    /*
    pub fn pos(&self) -> u64 {
        self.pos
    }
     */

    pub fn data_pos(&self) -> usize {
        self.data.start
    }

    pub fn data(&self) -> &'a [u8] {
        self.jpeg.data.get(self.data.clone()).unwrap()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Invalid magic bytes: {0:x?}")]
    InvalidMagicBytes([u8; 2]),
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Expected marker start: {0:x}")]
    ExpectedMarkerStart(u8),
}

gufo_common::utils::convertible_enum!(
    #[repr(u8)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    pub enum Marker {
        SOF0 = 0xC0,
        SOF1 = 0xC1,
        SOF2 = 0xC2,
        /// Define Huffman table
        DHT = 0xC4,
        /// Start of scan
        SOS = 0xDA,
        DQT = 0xDB,
        /// Start of image
        SOI = 0xd8,
        APP0 = 0xE0,
        /// Exif, XMP
        APP1 = 0xE1,
        /// ICC color profile
        APP2 = 0xE2,
        APP3 = 0xE3,
        APP4 = 0xE4,
        APP5 = 0xE5,
        APP6 = 0xE6,
        APP7 = 0xE7,
        APP8 = 0xE8,
        APP9 = 0xE9,
        APP10 = 0xEA,
        APP11 = 0xEB,
        APP12 = 0xEC,
        APP13 = 0xED,
        APP14 = 0xEE,
        APP15 = 0xEF,
        /// Define Restart Interval
        DRI = 0xDD,
        /// Comment
        COM = 0xFE,
    }
);
