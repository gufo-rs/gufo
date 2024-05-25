#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagIfd {
    pub tag: Tag,
    pub ifd: Ifd,
}

impl TagIfd {
    pub fn new(tag: Tag, ifd: Ifd) -> Self {
        Self { tag, ifd }
    }
}

impl<T: Field> From<T> for TagIfd {
    fn from(_value: T) -> Self {
        TagIfd {
            tag: T::TAG,
            ifd: T::IFD,
        }
    }
}

pub trait Field {
    const NAME: &'static str;
    const TAG: Tag;
    const IFD: Ifd;
    const IFD_ENTRY: Option<Ifd> = None;
}

pub fn lookup_tag_name(tagifd: TagIfd) -> Option<&'static str> {
    crate::field::TAG_NAMES
        .get(&(tagifd.tag.0, tagifd.ifd))
        .copied()
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Tag(pub u16);

impl Tag {
    pub const MAKER_NOTE: Self = Self(0x927C);

    pub const EXIF_IFD_POINTER: Self = Self(0x8769);
    pub const GPS_INFO_IFD_POINTER: Self = Self(0x8825);
    pub const INTEROPERABILITY_IFD_POINTER: Self = Self(0xA005);

    /// See 4.6.3 in v3.0 standard
    pub fn exif_specific_ifd(&self) -> Option<Ifd> {
        match *self {
            Self::EXIF_IFD_POINTER => Some(Ifd::Exif),
            Self::GPS_INFO_IFD_POINTER => Some(Ifd::Gps),
            Self::INTEROPERABILITY_IFD_POINTER => Some(Ifd::Interoperability),
            _ => None,
        }
    }

    pub fn is_exif_specific_ifd(&self) -> bool {
        self.exif_specific_ifd().is_some()
    }
}

/// Image file directory
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ifd {
    Primary,
    Thumbnail,
    Exif,
    Gps,
    Interoperability,
    MakerNote,
}
