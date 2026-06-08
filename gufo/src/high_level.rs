use gufo_common::types::Rational;
use gufo_common::{geography, hardware, orientation};
use gufo_exif::Exif;
use gufo_xmp::Xmp;

use crate::Metadata;

impl Metadata {
    /// Owner of the camera used in photography
    pub fn camera_owner_name(&self) -> Option<String> {
        self.exif_xmp(Exif::camera_owner_name, Xmp::camera_owner_name)
    }

    /// Name of the main person who created the image
    pub fn creator(&self) -> Option<String> {
        self.exif_xmp(Exif::artist, Xmp::creator)
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.exif_xmp(Exif::date_time_original, Xmp::date_time_original)
    }

    pub fn digital_zoom_ratio(&self) -> Option<Rational<u32>> {
        self.exif_xmp(Exif::digital_zoom_ratio, Xmp::digital_zoom_ratio)
    }

    /// Exposure time in seconds
    ///
    /// Fraction of first element devided by second element. The first element
    /// is typically one, such that the value is given in its common for like
    /// "1/60 sec".
    pub fn exposure_time(&self) -> Option<Rational<u32>> {
        self.exif_xmp(Exif::exposure_time, Xmp::exposure_time)
    }

    /// Aperture
    pub fn f_number(&self) -> Option<f32> {
        self.exif_xmp(|x| Exif::f_number(x).map(|x| x.as_f32()), Xmp::f_number)
    }

    /// Focal length in millimeters
    pub fn focal_length(&self) -> Option<Rational<u32>> {
        self.exif_xmp(Exif::focal_length, Xmp::focal_length)
    }

    pub fn gps_location(&self) -> Option<geography::Location> {
        // TODO: XMP
        self.get_exif(Exif::gps_location)
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.exif_xmp(Exif::iso_speed_rating, Xmp::iso_speed_rating)
    }

    pub fn lens_make(&self) -> Option<String> {
        self.exif_xmp(Exif::lens_make, Xmp::lens_make)
    }

    pub fn lens_model(&self) -> Option<String> {
        self.exif_xmp(Exif::lens_model, Xmp::lens_model)
    }

    pub fn lens_specification(&self) -> Option<hardware::LensSpecification> {
        // TODO: XMP
        self.get_exif(Exif::lens_specification)
    }

    /// Camera manifacturer
    pub fn make(&self) -> Option<String> {
        self.exif_xmp(Exif::make, Xmp::make)
    }

    /// Camera model
    pub fn model(&self) -> Option<String> {
        self.exif_xmp(Exif::model, Xmp::model)
    }

    /// Image orientation
    ///
    /// Rotation and mirroring that have to be applied to show the image
    /// correctly
    pub fn orientation(&self) -> Option<orientation::Orientation> {
        self.exif_xmp(Exif::orientation, Xmp::orientation)
    }

    /// Copyright information
    pub fn rights(&self) -> Option<String> {
        self.exif_xmp(Exif::copyright, Xmp::rights)
    }

    /// URL with usage rights information
    pub fn rights_web_statement(&self) -> Option<String> {
        self.get_xmp(Xmp::rights_web_statement)
    }

    /// Name and version of software or firmware
    ///
    /// In practice, this often contains the name, version, and operating system
    /// of the image editing software used to edit an image.
    pub fn software(&self) -> Option<String> {
        self.exif_xmp(Exif::software, Xmp::creator_tool)
    }

    /// Freely write keywords or comments on the image
    pub fn user_comment(&self) -> Option<String> {
        self.get_exif(Exif::user_comment)
    }
}
