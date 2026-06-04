use gufo_common::{geography, orientation};

use crate::structure::{Document, Rational};
use crate::{Error, ExifInternal, Storage};

impl<'a, S: Storage<'a>> ExifInternal<'a, S> {
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        self.document(|x| x.serialize())
    }

    pub fn camera_owner_name(&self) -> Option<String> {
        self.document(|x| x.camera_owner_name())
    }

    pub fn document<T>(&self, f: impl FnOnce(&mut Document<'_>) -> T) -> T {
        self.document.access(|x| f(x))
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.document(|x| x.date_time_original())
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
    pub fn f_number(&self) -> Option<f32> {
        self.document(|x| x.f_number())
    }

    /// Focal length in mm
    pub fn focal_length(&self) -> Option<f32> {
        self.document(|x| x.focal_length())
    }

    pub fn gps_location(&self) -> Option<geography::Location> {
        self.document(|x| x.gps_location())
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.document(|x| x.iso_speed_rating())
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

    pub fn software(&self) -> Option<String> {
        self.document(|x| x.software())
    }

    pub fn user_comment(&self) -> Option<String> {
        self.document(|x| x.user_comment())
    }
}
