use gufo_common::isobmff;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::combinator::map_res;
use nom::sequence::tuple;
use nom::IResult;

pub const MAGIC_BYTES_PLAIN: &[u8] = &[0xFF, 0x0A];
pub const MAGIC_BYTES_ISOBMFF: &[u8] = &[
    0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
];

pub struct Document<'a> {
    pub document: isobmff::Document<'a>,
}

impl<'a> Document<'a> {
    pub fn new(data: &'a [u8]) -> Result<Document, ()> {
        if let Some(isobmff) = Self::is_isobmff(data) {
            let document = isobmff::Document::new(data, MAGIC_BYTES_ISOBMFF.len() as u64);

            Ok(Self { document })
        } else {
            panic!("invalid file");
        }
    }

    pub fn is_isobmff(data: &'a [u8]) -> Option<bool> {
        if data.get(0..MAGIC_BYTES_PLAIN.len()) == Some(MAGIC_BYTES_PLAIN) {
            Some(false)
        } else if data.get(0..MAGIC_BYTES_ISOBMFF.len()) == Some(MAGIC_BYTES_ISOBMFF) {
            Some(true)
        } else {
            None
        }
    }

    pub fn image_data(&self) -> Result<DataIter<'a>, ()> {
        let complete = self
            .document
            .boxes_type(gufo_common::isobmff::BoxType::JxlImage)
            .next();

        if let Some(complete) = complete {
            let part = (&complete.data()[4..], complete.data_pos());
            Ok(DataIter::new(vec![part]))
        } else {
            let parts = self
                .document
                .boxes_type(gufo_common::isobmff::BoxType::JxlImagePartial)
                .map(|x| (&x.data()[4..], x.data_pos()))
                .collect::<Vec<(&'a [u8], u64)>>();

            Ok(DataIter::new(parts))
        }
    }
}

/**
```
# use gufo_jxl::{DataIter,Dist};

let mut d = DataIter::new(vec![(&[5], 0)]);
assert_eq!(5, d.read_bytes_u8::<8>().unwrap());

let mut d = DataIter::new(vec![(&[0b011101], 0)]);
assert_eq!(7, d.read_u32_d(Dist::Bits(2), Dist::Bits(4), Dist::Bits(6), Dist::Bits(8)).unwrap());

let mut d = DataIter::new(vec![(&[0b011101, 0], 0)]);
assert_eq!(7, d.read_u32_d(Dist::Bits(2), Dist::Bits(4), Dist::Bits(6), Dist::Bits(8)).unwrap());

let mut d = DataIter::new(vec![(&[0b011101, 0], 0)]);
assert_eq!(7, d.read_u32_d(Dist::Bits(2), Dist::Bits(4), Dist::Bits(6), Dist::Bits(8)).unwrap());
```
 */
pub struct DataIter<'a> {
    parts: Vec<(&'a [u8], u64)>,
    active_part: usize,
    offset_bytes: usize,
    offset_bits: usize,
}

impl<'a> Iterator for DataIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let part = self.parts.get(self.active_part)?.0;
        let result = part.get(self.offset_bytes).copied()?;
        self.offset_bytes += 1;

        if self.offset_bytes >= part.len() {
            self.offset_bytes = 0;
            self.active_part += 1;
        }

        Some(result)
    }
}

impl<'a> DataIter<'a> {
    pub fn new(parts: Vec<(&'a [u8], u64)>) -> Self {
        Self {
            parts,
            active_part: 0,
            offset_bytes: 0,
            offset_bits: 0,
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        let part = self.parts.get(self.active_part)?.0;
        part.get(self.offset_bytes).copied()
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        self.next().ok_or(Error::UnexpectedEOF)
    }

    #[inline]
    pub fn read_byte(&mut self) -> Result<u8, Error> {
        let byte = self.peek().ok_or(Error::UnexpectedEOF)?;
        //eprintln!("byte: {:b}", byte);
        let bit = (byte & (1 << self.offset_bits)) != 0;
        if self.offset_bits < 7 {
            self.offset_bits += 1;
        } else {
            self.offset_bytes += 1;
            self.offset_bits = 0;
        }
        Ok(if bit { 1 } else { 0 })
    }

    pub fn read_bytes_u8<const N: u8>(&mut self) -> Result<u8, Error> {
        let mut v = 0;
        for i in 0..N {
            v += self.read_byte()? << i;
        }
        Ok(v)
    }

    pub fn read_bytes_u32(&mut self, n: u8) -> Result<u32, Error> {
        let mut v = 0;
        for i in 0..n {
            v += (self.read_byte()? as u32) << i;
            dbg!(v);
        }
        Ok(v)
    }

    pub fn read_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_byte()? == 1)
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        Ok(0)
    }

    pub fn read_u32_d(&mut self, d0: Dist, d1: Dist, d2: Dist, d3: Dist) -> Result<u32, Error> {
        let dist = dbg!(self.read_bytes_u8::<2>()?);
        match dbg!(dist) {
            0 => d0.read(self),
            1 => d1.read(self),
            2 => d2.read(self),
            3 => d3.read(self),
            _ => panic!(),
        }
    }
}

pub enum Dist {
    Val(u32),
    Bits(u8),
}

impl Dist {
    pub fn read(self, data: &mut DataIter) -> Result<u32, Error> {
        match self {
            Self::Val(val) => Ok(val),
            Self::Bits(n) => dbg!(data.read_bytes_u32(n)),
        }
    }
}

pub struct Entry<T>(T, u64);

#[derive(Debug)]
pub struct Header {
    signature: Signature,
    size: SizeHeader,
    metadata: ImageMetadata,
}

impl Parse for Header {
    fn parse(data: &mut DataIter) -> Result<Self, Error> {
        let signature = dbg!(Signature::parse(data)?);
        let size = dbg!(SizeHeader::parse(data)?);
        let metadata = ImageMetadata::parse(data)?;

        Ok(Self {
            signature,
            size,
            metadata,
        })
    }
}

#[derive(Debug)]
pub struct Signature;

impl Parse for Signature {
    fn parse(data: &mut DataIter) -> Result<Self, Error> {
        if data.read_bytes_u8::<8>()? != 255 || data.read_bytes_u8::<8>()? != 10 {
            Err(Error::InvalidSignature)
        } else {
            Ok(Self)
        }
    }
}

#[derive(Debug)]
pub struct SizeHeader {
    height_div8_minus_1: Option<u8>,
    height_minus_1: Option<u32>,
    ratio: u8,
    width_div8_minus_1: Option<u8>,
    width_minus_1: Option<u32>,
}

impl Parse for SizeHeader {
    fn parse(data: &mut DataIter) -> Result<Self, Error> {
        let small = data.read_bool()?;

        let mut height_div8_minus_1 = None;
        let mut height_minus_1 = None;
        let mut width_div8_minus_1 = None;
        let mut width_minus_1 = None;

        if small {
            height_div8_minus_1 = Some(data.read_bytes_u8::<5>()?);
        } else {
            height_minus_1 = Some(data.read_u32_d(
                Dist::Bits(9),
                Dist::Bits(13),
                Dist::Bits(18),
                Dist::Bits(30),
            )?);
        }
        dbg!(height_minus_1);

        let ratio = data.read_bytes_u8::<3>()?;
        dbg!(ratio);

        if ratio == 0 {
            if small {
                width_div8_minus_1 = Some(data.read_bytes_u8::<5>()?);
            } else {
                width_minus_1 = Some(data.read_u32_d(
                    Dist::Bits(9),
                    Dist::Bits(13),
                    Dist::Bits(18),
                    Dist::Bits(30),
                )?);
            }
        }

        Ok(Self {
            height_div8_minus_1,
            height_minus_1,
            ratio,
            width_div8_minus_1,
            width_minus_1,
        })
    }
}

#[derive(Debug)]
pub struct ImageMetadata {
    orientation_minus_1: Option<u8>,
}

impl Parse for ImageMetadata {
    fn parse(data: &mut DataIter) -> Result<Self, Error> {
        let all_default = data.read_bool()?;
        dbg!(all_default);
        let extra_fields = if all_default {
            false
        } else {
            data.read_bool()?
        };
        dbg!(extra_fields);

        let mut orientation_minus_1 = None;
        if extra_fields {
            orientation_minus_1 = Some(data.read_bytes_u8::<3>()?);
        }

        Ok(Self {
            orientation_minus_1,
        })
    }
}

pub trait Parse: Sized {
    fn parse(data: &mut DataIter) -> Result<Self, Error>;
}

#[derive(Debug)]
pub enum Error {
    UnexpectedEOF,
    InvalidSignature,
}
