use gufo_common::types::Rational;
use gufo_common::{geography, hardware, orientation, physical_dimension};

use crate::structure::Document;
use crate::{Error, Exif, Storage};

impl<'a, S: Storage<'a>> Exif<'a, S> {
    /// Generate raw exif data representing the Exif data
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        self.document(|x| x.serialize())
    }

    /// Access to the underlying [`Document`]
    pub fn document<T>(&self, f: impl FnOnce(&mut Document<'_>) -> T) -> T {
        self.document.access(|x| f(x))
    }

    /// Name of the main person who created the image
    pub fn artist(&self) -> Option<String> {
        self.document(|x| x.artist())
    }

    /// Owner of the camera used in photography
    pub fn camera_owner_name(&self) -> Option<String> {
        self.document(|x| x.camera_owner_name())
    }

    /// Copyright information
    pub fn copyright(&self) -> Option<String> {
        self.document(|x| x.copyright())
    }

    #[cfg(feature = "chrono")]
    /// The date and time when the original image data was generated
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.document(|x| x.date_time_original())
    }

    pub fn digital_zoom_ratio(&self) -> Option<Rational<u32>> {
        self.document(|x| x.digital_zoom_ratio())
    }

    /// Exposure time in seconds
    ///
    /// Fraction of first element devided by second element. The first element
    /// is typically one, such that the value is given in its common for like
    /// "1/60 sec".
    pub fn exposure_time(&self) -> Option<Rational<u32>> {
        self.document(|x| x.exposure_time())
    }

    /// Aperture
    pub fn f_number(&self) -> Option<Rational<u32>> {
        self.document(|x| x.f_number())
    }

    /// Focal length in mm
    pub fn focal_length(&self) -> Option<Rational<u32>> {
        self.document(|x| x.focal_length())
    }

    /// GPS location
    pub fn gps_location(&self) -> Option<geography::Location> {
        self.document(|x| x.gps_location())
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.document(|x| x.iso_speed_rating())
    }

    pub fn lens_make(&self) -> Option<String> {
        self.document(|x| x.lens_make())
    }

    pub fn lens_model(&self) -> Option<String> {
        self.document(|x| x.lens_model())
    }

    pub fn lens_specification(&self) -> Option<hardware::LensSpecification> {
        self.document(|x| x.lens_specification())
    }

    /// Camera manifacturer
    pub fn make(&self) -> Option<String> {
        self.document(|x| x.make())
    }

    /// Camera model
    pub fn model(&self) -> Option<String> {
        self.document(|x| x.model())
    }

    /// Image orientation
    ///
    /// Rotation and mirroring that have to be applied to show the image
    /// correctly
    pub fn orientation(&self) -> Option<orientation::Orientation> {
        self.document(|x| x.orientation())
    }

    /// Resolution (Pixel Density)
    ///
    /// Number of pixels per physical unit like DPI
    pub fn resolution(&self) -> Option<physical_dimension::PixelDensity> {
        self.document(|x| x.resolution())
    }

    /// Name and version of software or firmware
    ///
    /// In practice, this often contains the name, version, and operating system
    /// of the image editing software used to edit an image.
    pub fn software(&self) -> Option<String> {
        self.document(|x| x.software())
    }

    /// Freely write keywords or comments on the image
    pub fn user_comment(&self) -> Option<String> {
        self.document(|x| x.user_comment())
    }

    /// Thumbnail image as raw JPEG data
    pub fn thumbnail(&self) -> Option<Vec<u8>> {
        self.document(|x| x.thumbnail())
    }
}
