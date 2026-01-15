#![doc = include_str!("../README.md")]

#[cfg(feature = "encoder")]
mod encoder;
mod segments;

use std::io::{Cursor, Read};
use std::ops::Range;

use gufo_common::error::ErrorWithData;
use gufo_common::math::*;
use gufo_common::prelude::*;
use indexmap::IndexMap;
pub use segments::*;

pub const EXIF_IDENTIFIER_STRING: &[u8] = b"Exif\0\0";
pub const XMP_IDENTIFIER_STRING: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

pub const MAGIC_BYTES: &[u8] = &[0xFF, 0xD8, 0xFF];

pub const MARKER_START: u8 = 0xFF;

#[derive(Debug)]
pub struct Jpeg {
    segments: Vec<RawSegment>,
    data: Vec<u8>,
}

impl ImageFormat for Jpeg {
    fn is_filetype(data: &[u8]) -> bool {
        data.starts_with(MAGIC_BYTES)
    }
}

impl ImageMetadata for Jpeg {
    fn exif(&self) -> Vec<Vec<u8>> {
        self.exif_data().map(|x| x.to_vec()).collect()
    }

    fn xmp(&self) -> Vec<Vec<u8>> {
        self.exif_data().map(|x| x.to_vec()).collect()
    }
}

impl Jpeg {
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        match Self::find_segments(&data) {
            Ok(segments) => Ok(Self { segments, data }),
            Err(err) => Err(ErrorWithData::new(err, data)),
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// List all segments in their order of appearance
    pub fn segments(&self) -> Vec<Segment<'_>> {
        self.segments.iter().map(|x| x.segment(self)).collect()
    }

    /// List all segments with the given marker
    pub fn segments_marker(&self, marker: Marker) -> impl Iterator<Item = Segment<'_>> {
        self.segments
            .iter()
            .filter(move |x| x.marker == Some(marker))
            .map(|x| x.segment(self))
    }

    /// Quantization tables with `Tq` value as key
    pub fn dqts(&self) -> Result<IndexMap<u8, Dqt>, Error> {
        let segments = self.segments();

        let mut dqts = Vec::new();
        for i in segments
            .into_iter()
            .filter(|x| x.marker == Some(Marker::DQT))
        {
            let data = i.data();
            dqts.push(Dqt::from_data(data)?);
        }

        let mut map = IndexMap::new();
        for dqt in dqts.into_iter().flatten() {
            map.insert(dqt.tq(), dqt);
        }

        Ok(map)
    }

    pub fn sof(&self) -> Result<Sof, Error> {
        let segment = self
            .segments()
            .into_iter()
            .find(|x| x.marker.is_some_and(|x| x.is_sof()))
            .ok_or(Error::NoSofSegmentFound)?;

        Sof::from_data(segment.data())
    }

    pub fn is_progressive(&self) -> Result<bool, Error> {
        let sof_marker = self
            .segments()
            .into_iter()
            .flat_map(|x| x.marker())
            .find(|x| x.is_sof())
            .ok_or(Error::NoSofSegmentFound)?;

        sof_marker.is_progressive_sof()
    }

    /// Number of SOS segments
    ///
    /// For `is_progressive()` being true, this is the number of scans.
    pub fn n_sos(&self) -> usize {
        self.segments()
            .into_iter()
            .filter(|x| matches!(x.marker, Some(Marker::SOS)))
            .count()
    }

    pub fn sos(&self) -> Result<Sos, Error> {
        let segment = self
            .segment_by_marker(Marker::SOS)
            .ok_or(Error::NoSosSegmentFound)?;

        Sos::from_data(segment.data())
    }

    pub fn components_specification_parameters(
        &self,
        component: usize,
    ) -> Result<ComponentSpecificationParameters, Error> {
        let cs = self
            .sos()?
            .components_specifications
            .get(component)
            .ok_or(Error::MissingComponentSpecification)?
            .cs;
        self.sof()?
            .parameters
            .iter()
            .find(|x| x.c == cs)
            .ok_or(Error::MissingComponentSpecificationParameters)
            .cloned()
    }

    pub fn color_model(&self) -> Result<ColorModel, Error> {
        let sof = self.sof()?;
        let n_components = sof.parameters.len();

        if let Some(app14) = self.segment_by_marker(Marker::APP14) {
            if app14.data().starts_with(b"Adobe\0") {
                if let Some(color_model) = app14.data().get(11) {
                    return match *color_model {
                        0 if n_components == 4 => Ok(ColorModel::Cmyk),
                        0 if n_components == 3 => Ok(ColorModel::Rgb),
                        1 => Ok(ColorModel::YCbCr),
                        2 => Ok(ColorModel::Ycck),
                        _ => Err(Error::UnknownColorModel),
                    };
                }
            }
        }

        match n_components {
            1 => Ok(ColorModel::Grayscale),
            3 => Ok(ColorModel::YCbCr),
            _ => Err(Error::UnknownColorModel),
        }
    }

    pub fn segment_by_marker(&self, marker: Marker) -> Option<Segment<'_>> {
        self.segments
            .iter()
            .find(|x| x.marker == Some(marker))
            .map(|x| x.segment(self))
    }

    pub fn exif_segments(&self) -> impl Iterator<Item = Segment<'_>> {
        self.segments_marker(Marker::APP1)
            .filter(|x| x.data().starts_with(EXIF_IDENTIFIER_STRING))
    }

    pub fn exif_data(&self) -> impl Iterator<Item = &[u8]> {
        self.exif_segments()
            .filter_map(|x| x.data().get(EXIF_IDENTIFIER_STRING.len()..))
    }

    pub fn xmp_segments(&self) -> impl Iterator<Item = Segment<'_>> {
        self.segments_marker(Marker::APP1)
            .filter(|x| x.data().starts_with(XMP_IDENTIFIER_STRING))
    }

    pub fn xmp_data(&self) -> impl Iterator<Item = &[u8]> {
        self.xmp_segments()
            .filter_map(|x| x.data().get(XMP_IDENTIFIER_STRING.len()..))
    }

    fn find_segments(data: &[u8]) -> Result<Vec<RawSegment>, Error> {
        let mut cur = Cursor::new(data);

        let buf = &mut [0; 2];
        cur.read_exact(buf).map_err(|_| Error::UnexpectedEof)?;

        if data.get(..MAGIC_BYTES.len()) != Some(MAGIC_BYTES) {
            return Err(Error::InvalidMagicBytes(*buf));
        }

        let mut segments = Vec::new();
        segments.push(RawSegment {
            marker: Some(Marker::SOI),
            data: 2..2,
        });

        let mut entropy_coded_segment = false;
        let byte = &mut [0; 1];
        loop {
            if entropy_coded_segment {
                let data_start = cur.position().usize()?;
                loop {
                    cur.read_exact(byte).map_err(|_| Error::UnexpectedEof)?;
                    if byte == &[MARKER_START] {
                        cur.read_exact(byte).map_err(|_| Error::UnexpectedEof)?;

                        if byte == &[0] {
                            continue;
                        } else {
                            let data_end = cur.position().safe_sub(2)?.usize()?;
                            segments.push(RawSegment {
                                marker: None,
                                data: data_start..data_end,
                            });
                            break;
                        }
                    }
                }
            } else {
                // Read tag
                cur.read_exact(byte).map_err(|_| Error::UnexpectedEof)?;

                if byte != &[MARKER_START] {
                    return Err(Error::ExpectedMarkerStart(buf[0]));
                }

                cur.read_exact(byte).map_err(|_| Error::UnexpectedEof)?;

                tracing::debug!("Found tag {byte:0>2X?}");
            }

            let marker = Marker::from(byte[0]);
            let len_start = cur.position();

            let (data_start, len) = if marker.is_standalone() {
                (len_start.usize()?, 0)
            } else {
                // Read length. The length includes the two length bytes, but not the marker.
                cur.read_exact(buf).map_err(|_| Error::UnexpectedEof)?;
                (len_start.usize()?.safe_add(2)?, u16::from_be_bytes(*buf))
            };

            let data_end = len_start.usize()?.safe_add(len.into())?;

            let segment = RawSegment {
                marker: Some(marker),
                data: data_start..data_end,
            };

            tracing::debug!("Found segment {segment:?}");

            segments.push(segment);

            if marker == Marker::EOI {
                break;
            } else if marker == Marker::SOS {
                entropy_coded_segment = true;
            }

            cur.set_position(len_start.safe_add(len.into())?);
        }

        Ok(segments)
    }

    pub fn replace_segment(
        &mut self,
        old_segment: RawSegment,
        new_segment: NewSegment,
    ) -> Result<(), Error> {
        let old_range = old_segment.complete_data();

        let mut new = Vec::new();
        new.extend_from_slice(&self.data[..old_range.start]);
        new_segment.write_to(&mut new);
        new.extend_from_slice(&self.data[old_range.end..]);

        self.data = new;
        self.segments = Self::find_segments(&self.data)?;
        Ok(())
    }

    /// Replaces this PNG's image data with those from another
    ///
    /// Keeps all the metadata from this image but replaces the `IHDR` and
    /// `IDAT` chunks with the ones from `other`.
    pub fn replace_image_data(&mut self, other: &Self) -> Result<(), Error> {
        let mut buf = Vec::with_capacity(other.data.len());
        buf.extend_from_slice(&MAGIC_BYTES[0..2]);

        for segment in &self.segments {
            if segment.marker.is_some_and(|x| x.is_metadata()) {
                buf.extend_from_slice(&self.data[segment.complete_data()]);
            }
        }

        for segment in &other.segments {
            if !matches!(segment.marker, Some(Marker::SOI)) {
                buf.extend_from_slice(&other.data[segment.complete_data()]);
            }
        }

        self.segments = Self::find_segments(&buf).unwrap();
        self.data = buf;

        Ok(())
    }
}

#[derive(Debug)]
pub struct NewSegment<'a> {
    marker: Marker,
    data: &'a [u8],
    total_len: u16,
}

impl<'a> NewSegment<'a> {
    pub fn new(marker: Marker, data: &'a [u8]) -> Result<Self, Error> {
        let total_len = data.len().u16()?.safe_add(2)?;

        Ok(Self {
            marker,
            data,
            total_len,
        })
    }

    pub fn write_to(&self, vec: &mut Vec<u8>) {
        vec.push(MARKER_START);
        vec.push(self.marker.into());
        vec.extend_from_slice(&self.total_len.to_be_bytes());
        vec.extend_from_slice(self.data);
    }
}

#[derive(Debug)]
pub struct RawSegment {
    marker: Option<Marker>,
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

    /// Complete segment including marker and length
    pub fn complete_data(&self) -> Range<usize> {
        let sub = if self.marker.is_some() { 4 } else { 0 };

        self.data
            .start
            .checked_sub(sub)
            .expect("Unreachable: Marker and length fields always exist")..self.data.end
    }
}

#[derive(Clone, Debug)]
pub struct Segment<'a> {
    marker: Option<Marker>,
    data: Range<usize>,
    jpeg: &'a Jpeg,
}

impl<'a> Segment<'a> {
    pub fn marker(&self) -> Option<Marker> {
        self.marker
    }

    pub fn data_pos(&self) -> usize {
        self.data.start
    }

    pub fn data(&self) -> &'a [u8] {
        self.jpeg
            .data
            .get(self.data.clone())
            .expect("Unreachable: This data must exist after successful loading")
    }

    pub fn unsafe_raw_segment(self) -> RawSegment {
        RawSegment {
            data: self.data,
            marker: self.marker,
        }
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
    #[error("Math error: {0}")]
    Math(#[from] MathError),
    #[error("Unknown uantization table element precision {0}")]
    UnknownPq(u8),
    #[error("No SOS segment found")]
    NoSosSegmentFound,
    #[error("No SOF segment found")]
    NoSofSegmentFound,
    #[error("Couldn't detemine a color model")]
    UnknownColorModel,
    #[error("Missing component specification")]
    MissingComponentSpecification,
    #[error("Missing component specification parameters")]
    MissingComponentSpecificationParameters,
    #[error("Missing quantization table")]
    MissingDqt,
}

gufo_common::utils::convertible_enum!(
    #[repr(u8)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    /// Segment marker
    pub enum Marker {
        TEM = 0x01,

        SOF0 = 0xC0,
        SOF1 = 0xC1,
        SOF2 = 0xC2,
        /// Define Huffman table
        DHT = 0xC4,
        RST0 = 0xD0,
        RST1 = 0xD1,
        RST2 = 0xD2,
        RST3 = 0xD3,
        RST4 = 0xD4,
        RST5 = 0xD5,
        RST6 = 0xD6,
        RST7 = 0xD7,
        /// Start of image
        SOI = 0xD8,
        /// End of image
        EOI = 0xD9,
        /// Start of scan
        SOS = 0xDA,
        /// Define quantization table(s)
        DQT = 0xDB,

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

        JPG0 = 0xF0,
        JPG1 = 0xF1,
        JPG2 = 0xF2,
        JPG3 = 0xF3,
        JPG4 = 0xF4,
        JPG5 = 0xF5,
        JPG6 = 0xF6,
        JPG7 = 0xF7,
        JPG8 = 0xF8,
        JPG9 = 0xF9,
        JPG10 = 0xFA,
        JPG11 = 0xFB,
        JPG12 = 0xFC,
        JPG13 = 0xFD,
        /// Comment
        COM = 0xFE,
    }
);

impl Marker {
    pub fn is_standalone(&self) -> bool {
        matches!(
            self,
            Self::RST0
                | Self::RST1
                | Self::RST2
                | Self::RST3
                | Self::RST4
                | Self::RST5
                | Self::RST6
                | Self::RST7
                | Self::SOI
                | Self::EOI
        )
    }

    pub fn is_sof(&self) -> bool {
        matches!(self, Self::SOF0 | Self::SOF1 | Self::SOF2)
    }

    pub fn is_progressive_sof(&self) -> Result<bool, Error> {
        match self {
            Self::SOF0 | Self::SOF1 => Ok(false),
            Self::SOF2 => Ok(true),
            _ => Err(Error::NoSofSegmentFound),
        }
    }

    pub fn is_metadata(&self) -> bool {
        matches!(
            self,
            Self::COM
                | Self::APP0
                | Self::APP1
                | Self::APP2
                | Self::APP3
                | Self::APP4
                | Self::APP5
                | Self::APP6
                | Self::APP7
                | Self::APP8
                | Self::APP9
                | Self::APP10
                | Self::APP11
                | Self::APP12
                | Self::APP13
                | Self::APP14
                | Self::APP15
        )
    }
}

#[derive(Debug)]
pub enum ColorModel {
    Grayscale,
    YCbCr,
    Cmyk,
    Rgb,
    Ycck,
}
