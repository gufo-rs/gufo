/// XMP field
pub trait Field {
    /// XMP field name
    const NAME: &'static str;
    /// Set to true if value uses exifEX namespace
    ///
    /// If set to true, this field is from Exif 2.21 or later and uses the `http://cipa.jp/exif/1.0/` namespace (exifEX). Otherwise, it uses the legacy namespace `http://ns.adobe.com/exif/1.0/` (exif).
    const EX: bool = false;
}
