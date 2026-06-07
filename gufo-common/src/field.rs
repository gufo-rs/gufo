//! Metadata fields
//!
//! Definition of metadata fields that can be looked up from various formats.
//! Currently supported are Exif and XMP.

mod macros;

use crate::exif::IfdId;

// Exif
macros::make_tags![
    // Interoperability
    (0x1, InteroperabilityIndex, IfdId::Interoperability),
    (0x2, InteroperabilityVersion, IfdId::Interoperability),

    // GPS
    (0x0, GPSVersionID, IfdId::Gps, xmp = Exif),
    (0x1, GPSLatitudeRef, IfdId::Gps),
    (0x2, GPSLatitude, IfdId::Gps, xmp = Exif),
    (0x3, GPSLongitudeRef, IfdId::Gps),
    (0x4, GPSLongitude, IfdId::Gps, xmp = Exif),
    /// TODO: Xmp mapping unclear
    (0x5, GPSAltitudeRef, IfdId::Gps),
    (0x6, GPSAltitude, IfdId::Gps, xmp = Exif),
    (0x7, GPSTimeStamp, IfdId::Gps, xmp = Exif),
    (0x8, GPSSatellites, IfdId::Gps),
    (0x9, GPSStatus, IfdId::Gps),
    (0xA, GPSMeasureMode, IfdId::Gps),
    (0xB, GPSDOP, IfdId::Gps),
    (0xC, GPSSpeedRef, IfdId::Gps, xmp = Exif),
    (0xD, GPSSpeed, IfdId::Gps, xmp = Exif),
    (0x10, GPSImgDirectionRef, IfdId::Gps, xmp = Exif),
    (0x11, GPSImgDirection, IfdId::Gps, xmp = Exif),
    (0x12, GPSMapDatum, IfdId::Gps),
    (0x17, GPSDestBearingRef, IfdId::Gps, xmp = Exif),
    (0x18, GPSDestBearing, IfdId::Gps, xmp = Exif),
    (0x1D, GPSDateStamp, IfdId::Gps),
    (0x1F, GPSHPositioningError, IfdId::Gps, xmp = Exif),

    // Primary/Thumbnail
    (0x100, ImageWidth, IfdId::Primary, xmp = Exif),
    (0x100, ThumbnailImageWidth, IfdId::Thumbnail),
    (0x101, ImageHeight, IfdId::Primary, xmp = Exif),
    (0x102, BitsPerSample, IfdId::Primary, xmp = Tiff),
    (0x103, Compression, IfdId::Primary, xmp = Tiff),
    (0x106, PhotometricInterpretation, IfdId::Primary, xmp = Tiff),
    (0x10A, FillOrder, IfdId::Primary),
    (0x10E, ImageDescription, IfdId::Primary, xmp = Dc),
    (0x10F, Make, IfdId::Primary, xmp = Tiff),
    (0x110, Model, IfdId::Primary, xmp = Tiff),
    (0x111, StripOffsets, IfdId::Primary),
    /// Image orientation and mirroring
    (0x112, Orientation, IfdId::Primary, xmp = Tiff),
    (0x112, ThumbnailOrientation, IfdId::Thumbnail),
    (0x115, SamplesPerPixel, IfdId::Primary, xmp = Tiff),
    (0x116, RowsPerStrip, IfdId::Primary),
    (0x117, StripByteCounts, IfdId::Primary),
    (0x11A, XResolution, IfdId::Primary, xmp = Tiff),
    (0x11B, YResolution, IfdId::Primary, xmp = Tiff),
    (0x11C, PlanarConfiguration, IfdId::Primary, xmp = Tiff),
    (0x11E, XPosition, IfdId::Primary),
    (0x11F, YPosition, IfdId::Primary),
    (0x128, ResolutionUnit, IfdId::Primary, xmp = Tiff),
    (0x129, PageNumber, IfdId::Primary),
    /// The XMP equivalent is [`CreatorTool`]
    (0x131, Software, IfdId::Primary),
    /// The XMP equivalent is [`ModifyDate`]
    (0x132, DateTime, IfdId::Primary),
    (0x13B, Artist, IfdId::Primary),
    (0x13E, WhitePoint, IfdId::Primary, xmp = Tiff),
    (0x13F, PrimaryChromaticities, IfdId::Primary, xmp = Tiff),
    (0x213, YCbCrPositioning, IfdId::Primary),
    (0x258, Xmp, IfdId::Primary),
    (0x8298, Copyright, IfdId::Primary),


    // Exif
    (0x829A, ExposureTime, IfdId::Exif, xmp = Exif),
    (0x829D, FNumber, IfdId::Exif, xmp = Exif),
    /// Points to the start of the [`Ifd::Exif`] entries list
    (0x8769,  ExifIFDPointer, IfdId::Primary),
    (0x8822, ExposureProgram, IfdId::Exif, xmp = Exif),
    /// Points to the start of the [`Ifd::GPS`] entries list
    (0x8825, GPSInfoIFDPointer, IfdId::Primary),
    /// Also called ISOSpeedRatings (new xmp value since Exif 2.3 or later)
    (0x8827, PhotographicSensitivity, IfdId::Exif, xmp = ExifEX),
    (0x8830, SensitivityType, IfdId::Exif),
    (0x8832, RecommendedExposureIndex, IfdId::Exif),
    (0x9000, ExifVersion, IfdId::Exif, xmp = Exif),
    (0x9003, DateTimeOriginal, IfdId::Exif, xmp = Exif),
    (0x9004, DateTimeDigitized, IfdId::Exif, xmp = Exif),
    (0x9010, OffsetTime, IfdId::Exif),
    (0x9012, OffsetTimeDigitized, IfdId::Exif),
    (0x9011, OffsetTimeOriginal, IfdId::Exif),
    (0x9101, ComponentsConfiguration, IfdId::Exif, xmp = Exif),
    (0x9201, ShutterSpeedValue, IfdId::Exif, xmp = Exif),
    /// Lens aperture with unit APEX
    (0x9202, Aperture, IfdId::Exif, xmp = Exif),
    (0x9203, BrightnessValue, IfdId::Exif, xmp = Exif),
    (0x9204, ExposureBiasValue, IfdId::Exif, xmp = Exif),
    (0x9205, MaxApertureValue, IfdId::Exif),
    (0x9206, SubjectDistance, IfdId::Exif),
    (0x9207, MeteringMode, IfdId::Exif, xmp = Exif),
    (0x9208, LightSource, IfdId::Exif, xmp = Exif),
    (0x9209, Flash, IfdId::Exif),
    (0x920A, FocalLength, IfdId::Exif, xmp = Exif),
    (0x927C, MakerNote, IfdId::Exif),
    (0x9286, UserComment, IfdId::Exif, xmp = Exif),
    (0x9290, SubSecTime, IfdId::Exif),
    (0x9291, SubSecTimeOriginal, IfdId::Exif),
    (0x9292, SubsecTimeDigitized, IfdId::Exif, xmp = Exif),
    (0xA000, FlashpixVersion, IfdId::Exif, xmp = Exif),
    (0xA001, ColorSpace, IfdId::Exif, xmp = Exif),
    (0xA002, PixelXDimension, IfdId::Exif, xmp = Exif),
    (0xA003, PixelYDimension, IfdId::Exif, xmp = Exif),
    (0xA005, InteroperabilityIfd, IfdId::Exif),
    (0xA20E, FocalPlaneXResolution, IfdId::Exif),
    (0xA20F, FocalPlaneYResolution, IfdId::Exif),
    (0xA210, FocalPlaneResolutionUnit, IfdId::Exif),

    (0xA217, SensingMethod, IfdId::Exif, xmp = Exif),
    (0xA300, FileSource, IfdId::Exif, xmp = Exif),

    (0xA301, SceneType, IfdId::Exif, xmp = Exif),
    (0xA401, CustomRendered, IfdId::Exif),
    (0xA402, ExposureMode, IfdId::Exif, xmp = Exif),
    (0xA403, WhiteBalance, IfdId::Exif, xmp = Exif),
    (0xA404, DigitalZoomRatio, IfdId::Exif),
    (0xA405, FocalLengthIn35mmFilm, IfdId::Exif, xmp = Exif),
    (0xA406, SceneCaptureType, IfdId::Exif, xmp = Exif),
    (0xA407, GainControl, IfdId::Exif),
    (0xA408, Contrast, IfdId::Exif),
    (0xA409, Saturation, IfdId::Exif),
    (0xA40A, Sharpness, IfdId::Exif),
    (0xA40C, SubjectDistanceRange, IfdId::Exif),
    (0xA420, ImageUniqueID, IfdId::Exif),
    (0xA430, CameraOwnerName, IfdId::Exif, xmp = ExifEX),
    (0xA431, BodySerialNumber, IfdId::Exif),
    (0xA432, LensSpecification, IfdId::Exif, xmp = ExifEX),
    (0xA433, LensMake, IfdId::Exif),
    (0xA434, LensModel, IfdId::Exif),
    (0xA435, LensSerialNumber, IfdId::Exif),

    // Canon
    (0x7, CanonFirmwareVersion, IfdId::MakerNote),
    (0x9, CanonCameraOwnerName, IfdId::MakerNote),
    (0x95, CanonLensModel, IfdId::MakerNote),
];

macros::make_xmp_tags![
    /// Legacy XMP Exif (till Exif 2.21)
    (ISOSpeedRatings, Exif)
];

// Dublin Core
macros::make_xmp_tags![(Creator, "creator", Dc)];

// XMP
macros::make_xmp_tags![(CreatorTool, Xmp)];
macros::make_xmp_tags![(ModifyDate, Xmp)];
