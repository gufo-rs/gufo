//! Metadata fields
//!
//! Definition of metadata fields that can be looked up from various formats.
//! Currently supported are Exif and XMP.

mod macros;

use macros::*;

use crate::exif::Ifd;

macros::make_tags![
    // Primary
    (0x10F, Make, Ifd::Primary, xmp = old),
    (0x110, Model, Ifd::Primary, xmp = old),
    /// Image orientation and mirroring
    (0x112, Orientation, Ifd::Primary, xmp = old),
    (0x112, ThumbnailOrientation, Ifd::Thumbnail),
    (0x11A, XResolution, Ifd::Primary, xmp = old),
    (0x0100, ImageWidth, Ifd::Primary, xmp = old),
    (0x0100, ThumbnailImageWidth, Ifd::Thumbnail, xmp = old),
    // Exif
    (0x829A, ExposureTime, Ifd::Exif, xmp = old),
    (0x829D, FNumber, Ifd::Exif, xmp = old),
    (0x8827, PhotographicSensitivity, Ifd::Exif, xmp = new),
    (0x9003, DateTimeOriginal, Ifd::Exif, xmp = old),
    (0x9011, OffsetTimeOriginal, Ifd::Exif),
    (0x920A, FocalLength, Ifd::Exif, xmp = old),
    (0xA433, LensMake, Ifd::Exif, xmp = new),
    (0xA434, LensModel, Ifd::Exif, xmp = new),
];
