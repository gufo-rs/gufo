use std::ops::Deref;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagIfd {
    pub tag: Tag,
    pub ifd: IfdId,
}

impl TagIfd {
    pub fn new(tag: Tag, ifd: IfdId) -> Self {
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
    const IFD: IfdId;
    const IFD_ENTRY: Option<IfdId> = None;
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

    /// Returns the IFD if the tag stores the location of that IFD
    ///
    /// See 4.6.3 in v3.0 standard
    pub fn exif_specific_ifd(&self) -> Option<IfdId> {
        match *self {
            Self::EXIF_IFD_POINTER => Some(IfdId::Exif),
            Self::GPS_INFO_IFD_POINTER => Some(IfdId::Gps),
            Self::INTEROPERABILITY_IFD_POINTER => Some(IfdId::Interoperability),
            _ => None,
        }
    }

    pub fn is_exif_specific_ifd(&self) -> bool {
        self.exif_specific_ifd().is_some()
    }
}

impl Deref for Tag {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Image file directory
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum IfdId {
    Primary,
    Thumbnail,
    Exif,
    Gps,
    Interoperability,
    MakerNote,
}
