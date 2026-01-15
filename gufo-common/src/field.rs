//! Metadata fields
//!
//! Definition of metadata fields that can be looked up from various formats.
//! Currently supported are Exif and XMP.

mod macros;

use crate::exif::Ifd;

// Exif
macros::make_tags![
    // GPS
    (0x1, GPSLatitudeRef, Ifd::Gps),
    (0x2, GPSLatitude, Ifd::Gps),
    (0x3, GPSLongitudeRef, Ifd::Gps),
    (0x4, GPSLongitude, Ifd::Gps),
    (0x5, GPSAltitudeRef, Ifd::Gps),
    (0x6, GPSAltitude, Ifd::Gps),
    (0x10, GPSImgDirectionRef, Ifd::Gps),
    (0x11, GPSImgDirection, Ifd::Gps),
    (0x12, GPSSpeedRef, Ifd::Gps),
    (0x13, GPSSpeed, Ifd::Gps),

    // Primary
    (0x100, ImageWidth, Ifd::Primary, xmp = Exif),
    (0x10E, ImageDescription, Ifd::Primary),
    (0x10F, Make, Ifd::Primary, xmp = Tiff),
    (0x110, Model, Ifd::Primary, xmp = Tiff),
    /// Image orientation and mirroring
    (0x112, Orientation, Ifd::Primary, xmp = Tiff),
    (0x112, ThumbnailOrientation, Ifd::Thumbnail),
    (0x11A, XResolution, Ifd::Primary, xmp = Tiff),
    (0x11B, YResolution, Ifd::Primary, xmp = Tiff),
    (0x128, ResolutionUnit, Ifd::Primary, xmp = Tiff),
    /// The XMP equivalent is [`CreatorTool`]
    (0x131, Software, Ifd::Primary),

    // Thumbnail
    (0x100, ThumbnailImageWidth, Ifd::Thumbnail, xmp = Exif),

    // Exif
    (0x829A, ExposureTime, Ifd::Exif, xmp = Exif),
    (0x829D, FNumber, Ifd::Exif, xmp = Exif),
    /// Also called ISOSpeedRatings (new xmp value since Exif 2.3 or later)
    (0x8827, PhotographicSensitivity, Ifd::Exif, xmp = ExifEX),
    (0x9003, DateTimeOriginal, Ifd::Exif, xmp = Exif),
    (0x9011, OffsetTimeOriginal, Ifd::Exif),
    (0x9286, UserComment, Ifd::Exif, xmp = Exif),
    (0x9291, SubSecTimeOriginal, Ifd::Exif),
    /// Lens aperture with unit APEX
    (0x9202, Aperture, Ifd::Exif, xmp = Exif),
    (0x920A, FocalLength, Ifd::Exif, xmp = Exif),
    (0xA430, CameraOwnerName, Ifd::Exif, xmp = ExifEX),
    (0xA433, LensMake, Ifd::Exif, xmp = Exif),
    (0xA434, LensModel, Ifd::Exif, xmp = Exif),
];

macros::make_exif_tags!((0x9, CanonOwnerName, Ifd::MakerNote),);

macros::make_xmp_tags![
    /// Legacy XMP Exif (till Exif 2.21)
    (ISOSpeedRatings, Exif)
];

// Dublin Core
macros::make_xmp_tags![(Creator, "creator", Dc)];

// XMP
macros::make_xmp_tags![(CreatorTool, Xmp)];
