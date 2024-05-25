use std::io::{Cursor, Read, Seek, SeekFrom};

pub const EXIF_IDENTIFIER_STRING: &[u8] = b"Exif\0\0";
pub const XMP_IDENTIFIER_STRING: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

#[derive(Clone, Debug)]
pub struct Segment<'a> {
    marker: Marker,
    pos: u64,
    data: &'a [u8],
}

impl<'a> Segment<'a> {
    pub fn marker(&self) -> Marker {
        self.marker
    }

    pub fn pos(&self) -> u64 {
        self.pos
    }

    pub fn data_pos(&self) -> u64 {
        self.pos + 2
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

pub const MARKER_START: u16 = 0xFF;

pub struct Jpeg<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'a> Jpeg<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let segments = Self::find_segments(data);
        Self { segments }
    }

    /// List all segments in their order of appearance
    pub fn segments(&self) -> &'a [Segment] {
        &self.segments
    }

    /// List all segments with the given marker
    pub fn segments_marker(&self, marker: Marker) -> impl Iterator<Item = &Segment> {
        self.segments.iter().filter(move |x| x.marker == marker)
    }

    pub fn exif(&self) -> impl Iterator<Item = &Segment> {
        self.segments_marker(Marker::APP1)
            .filter(|x| x.data().starts_with(EXIF_IDENTIFIER_STRING))
    }

    pub fn exif_data(&self) -> impl Iterator<Item = &[u8]> {
        self.exif()
            .filter_map(|x| x.data().get(EXIF_IDENTIFIER_STRING.len()..))
    }

    pub fn xmp(&self) -> impl Iterator<Item = &Segment> {
        self.segments_marker(Marker::APP1)
            .filter(|x| x.data().starts_with(XMP_IDENTIFIER_STRING))
    }

    pub fn xmp_data(&self) -> impl Iterator<Item = &[u8]> {
        self.xmp()
            .filter_map(|x| x.data().get(XMP_IDENTIFIER_STRING.len()..))
    }

    fn find_segments(data: &[u8]) -> Vec<Segment> {
        let mut source = Cursor::new(data);

        let buf = &mut [0; 2];
        source.read_exact(buf).unwrap();

        eprintln!("{buf:x?}");

        let mut segments = Vec::new();
        loop {
            // Read tag
            source.read_exact(buf).unwrap();
            eprintln!("{buf:x?}");

            let marker = Marker::from(buf[1]);
            let pos = source.stream_position().unwrap();

            // Read len
            source.read_exact(buf).unwrap();
            let len: u16 = u16::from_be_bytes(*buf);

            let segment = Segment {
                marker,
                pos,
                data: data
                    .as_ref()
                    .get((pos + 2) as usize..(pos as usize + len as usize))
                    .unwrap(),
            };

            segments.push(segment);

            if marker == Marker::SOS {
                break;
            }
            source.seek(SeekFrom::Current(len as i64 - 2)).unwrap();
        }

        segments
    }
}

gufo_common::utils::convertible_enum!(
    #[repr(u8)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        /// Comment
        COM = 0xFE,
    }
);
