use std::fmt::Display;

use crate::structure::util::Endieness;
use crate::Error;

gufo_common::utils::convertible_enum!(
    #[repr(u16)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Exif datatype
    ///
    /// Specifies which type an entry has
    pub enum Type {
        Byte = 1,
        Ascii = 2,
        Short = 3,
        Long = 4,
        Rational = 5,
        Undefined = 7,
        SLong = 9,
        SRational = 10,
        Utf8 = 129,
    }
);

#[derive(Debug, Clone, Copy)]
pub struct Rational<T: Display> {
    pub numerator: T,
    pub denominator: T,
}

impl Rational<u32> {
    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }

    pub fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

impl Rational<i32> {
    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }
}

impl<T: Display> Rational<T> {
    pub fn display(&self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }
}

impl Type {
    /// Size of an entry per count
    pub const fn size(self) -> usize {
        match self {
            Self::Byte | Self::Ascii | Self::Undefined | Self::Utf8 | Self::Unknown(_) => 1,
            Self::Short => 2,
            Self::Long | Self::SLong => 4,
            Self::Rational | Self::SRational => 8,
        }
    }

    pub fn u16(self) -> u16 {
        self.into()
    }
}

#[derive(Debug)]
pub enum Typed {
    Byte(Vec<u8>),
    Ascii(Vec<u8>),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational(Vec<Rational<u32>>),
    Undefined(Vec<u8>),
    SLong(Vec<i32>),
    SRational(Vec<Rational<i32>>),
    Utf8(String),
    Unknown(u16, Vec<u8>),
}

impl Typed {
    pub fn new(
        type_: Type,
        count: usize,
        data: &[u8],
        endieness: Endieness,
    ) -> Result<Self, Error> {
        match type_ {
            Type::Ascii => {
                let mut data = data.to_vec();

                if let Some(last) = data.last() {
                    if *last == b'\0' {
                        data.pop();
                    }
                }

                Ok(Self::Ascii(data))
            }
            Type::Short => {
                let vec = data
                    .chunks_exact(2)
                    .take(count)
                    .map(|x| endieness.u16_from_bytes(x))
                    .collect::<Result<Vec<_>, Error>>()?;

                Ok(Self::Short(vec))
            }
            Type::Long => {
                let vec = data
                    .chunks_exact(4)
                    .take(count)
                    .map(|x| endieness.u32_from_bytes(x))
                    .collect::<Result<Vec<_>, Error>>()?;

                Ok(Self::Long(vec))
            }
            Type::Rational => {
                let vec = data
                    .chunks_exact(8)
                    .take(count)
                    .map(|x| {
                        let num = x.get(..4).ok_or(Error::IndexOverflow)?;
                        let den = x.get(4..).ok_or(Error::IndexOverflow)?;
                        Ok(Rational {
                            numerator: endieness.u32_from_bytes(num)?,
                            denominator: endieness.u32_from_bytes(den)?,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;

                Ok(Self::Rational(vec))
            }
            Type::Undefined => Ok(Self::Undefined(data.to_vec())),
            Type::SLong => {
                let vec = data
                    .chunks_exact(4)
                    .take(count)
                    .map(|x| endieness.i32_from_bytes(x))
                    .collect::<Result<Vec<_>, Error>>()?;

                Ok(Self::SLong(vec))
            }
            Type::SRational => {
                let vec = data
                    .chunks_exact(8)
                    .take(count)
                    .map(|x| {
                        let num = x.get(..4).ok_or(Error::IndexOverflow)?;
                        let den = x.get(4..).ok_or(Error::IndexOverflow)?;
                        Ok(Rational {
                            numerator: endieness.i32_from_bytes(num)?,
                            denominator: endieness.i32_from_bytes(den)?,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;

                Ok(Self::SRational(vec))
            }
            Type::Utf8 => {
                let mut s: String = String::from_utf8_lossy(data).to_string();

                if let Some(last) = s.bytes().last() {
                    if last == b'\0' {
                        s.pop();
                    }
                }

                Ok(Self::Utf8(s))
            }
            Type::Byte => Ok(Typed::Byte(data.to_vec())),
            Type::Unknown(type_id) => Ok(Typed::Unknown(type_id, data.to_vec())),
        }
    }

    pub fn type_(&self) -> Type {
        match self {
            Self::Byte(_) => Type::Byte,
            Self::Ascii(_) => Type::Ascii,
            Self::Short(_) => Type::Short,
            Self::Long(_) => Type::Long,
            Self::Rational(_) => Type::Rational,
            Self::Undefined(_) => Type::Undefined,
            Self::SLong(_) => Type::SLong,
            Self::SRational(_) => Type::SRational,
            Self::Utf8(_) => Type::Utf8,
            Self::Unknown(type_id, _) => Type::Unknown(*type_id),
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::Byte(data) => pp(data.iter()),
            Self::Ascii(data) => String::from_utf8_lossy(data).to_string(),
            Self::Short(data) => pp(data.iter()),
            Self::Long(data) => pp(data.iter()),
            Self::Rational(data) => pp(data.iter().map(|x| x.display())),
            Self::Undefined(data) => pp(data.iter()),
            Self::SLong(data) => pp(data.iter()),
            Self::SRational(data) => pp(data.iter().map(|x| x.display())),
            Self::Utf8(data) => data.clone(),
            Self::Unknown(_, data) => pp(data.iter()),
        }
    }
}

fn pp<T: Display>(data: impl Iterator<Item = T>) -> String {
    const MAX_DISPLAY: usize = 10;

    data.take(MAX_DISPLAY)
        .enumerate()
        .map(|(n, x)| {
            if n == MAX_DISPLAY - 1 {
                String::from("…")
            } else {
                x.to_string()
            }
        })
        .reduce(|x, y| format!("{x}, {y}"))
        .unwrap_or_default()
}
