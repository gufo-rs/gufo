gufo_common::utils::convertible_enum!(
    #[repr(u16)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn size(self) -> u32 {
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
