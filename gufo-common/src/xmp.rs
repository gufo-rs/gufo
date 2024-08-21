/// XMP field
pub trait Field {
    /// XMP field name
    const NAME: &'static str;
    /// Set to true if value uses exifEX namespace
    ///
    /// If set to true, this field is from Exif 2.21 or later and uses the `http://cipa.jp/exif/1.0/` namespace (exifEX). Otherwise, it uses the legacy namespace `http://ns.adobe.com/exif/1.0/` (exif).
    const NAMESPACE: Namespace;
}

/// Namespace for fields defined in TIFF
const XML_NS_TIFF: &str = "http://ns.adobe.com/tiff/1.0/";
/// Namespace for fields defined in Exif 2.2 or earlier
const XML_NS_EXIF: &str = "http://ns.adobe.com/exif/1.0/";
/// Namespace for fields defined in Exif 2.21 or later
const XML_NS_EXIF_EX: &str = "http://cipa.jp/exif/1.0/";

const XML_NS_XMP: &str = "http://ns.adobe.com/xap/1.0/";
const XML_NS_XMP_RIGHTS: &str = "http://ns.adobe.com/xap/1.0/rights/";
/// RDF
pub const XML_NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const XML_NS_PS: &str = "http://ns.adobe.com/photoshop/1.0/";
const XML_NS_DC: &str = "http://purl.org/dc/elements/1.1/";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Namespace {
    /// Namespace for fields defined in TIFF
    Tiff,
    /// Namespace for fields defined in Exif 2.2 or earlier
    Exif,
    /// Namespace for fields defined in Exif 2.21 or later
    ExifEX,
    Ps,
    Dc,
    Xmp,
    XmpRights,
    Unknown(String),
}

impl Namespace {
    pub fn from_url(url: &str) -> Self {
        match url {
            XML_NS_TIFF => Namespace::Tiff,
            XML_NS_EXIF => Namespace::Exif,
            XML_NS_EXIF_EX => Namespace::ExifEX,
            XML_NS_XMP => Namespace::Xmp,
            XML_NS_XMP_RIGHTS => Namespace::XmpRights,
            XML_NS_PS => Namespace::Ps,
            XML_NS_DC => Namespace::Dc,
            namespace => Namespace::Unknown(namespace.to_string()),
        }
    }

    pub fn to_url(&self) -> &str {
        match self {
            Namespace::Tiff => XML_NS_TIFF,
            Namespace::Exif => XML_NS_EXIF,
            Namespace::ExifEX => XML_NS_EXIF_EX,
            Namespace::Xmp => XML_NS_XMP,
            Namespace::XmpRights => XML_NS_XMP_RIGHTS,
            Namespace::Ps => XML_NS_PS,
            Namespace::Dc => XML_NS_DC,
            Namespace::Unknown(namespace) => namespace.as_str(),
        }
    }
}
