use std::fmt::Display;

use gufo_common::types::Rational;

use crate::Error;
use crate::structure::util::Endieness;

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

#[derive(Debug, PartialEq, Eq)]
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

    pub fn serialize(&self, endieness: Endieness) -> Vec<u8> {
        match self {
            Self::Ascii(data) => {
                let mut vec = data.to_vec();
                vec.push(0);
                vec
            }
            Self::Byte(data) => data.to_vec(),
            Self::Long(long) => long
                .iter()
                .flat_map(|x| endieness.u32_to_bytes(*x))
                .collect(),
            Self::Rational(rational) => rational
                .iter()
                .flat_map(|x| {
                    [
                        endieness.u32_to_bytes(x.numerator),
                        endieness.u32_to_bytes(x.denominator),
                    ]
                })
                .flatten()
                .collect(),
            Self::SLong(slong) => slong
                .iter()
                .flat_map(|x| endieness.i32_to_bytes(*x))
                .collect(),
            Self::SRational(srational) => srational
                .iter()
                .flat_map(|x| {
                    [
                        endieness.i32_to_bytes(x.numerator),
                        endieness.i32_to_bytes(x.denominator),
                    ]
                })
                .flatten()
                .collect(),
            Self::Short(short) => short
                .iter()
                .flat_map(|x| endieness.u16_to_bytes(*x))
                .collect(),
            Self::Undefined(undefined) => undefined.to_vec(),
            Self::Utf8(utf8) => {
                let mut vec = utf8.as_bytes().to_vec();
                vec.push(0);
                vec
            }
            Self::Unknown(_, unknown) => unknown.to_vec(),
        }
    }

    pub fn count(&self) -> usize {
        match self {
            Self::Ascii(x) => x.len() + 1,
            Self::Byte(x) => x.len(),
            Self::Long(x) => x.len(),
            Self::Rational(x) => x.len(),
            Self::SLong(x) => x.len(),
            Self::SRational(x) => x.len(),
            Self::Short(x) => x.len(),
            Self::Undefined(x) => x.len(),
            Self::Unknown(_, x) => x.len(),
            Self::Utf8(x) => x.len() + 1,
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
