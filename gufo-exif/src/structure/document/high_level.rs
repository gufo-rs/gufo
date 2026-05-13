use gufo_common::{field, geography, orientation};

use super::Document;
use crate::structure::util::{handle_error, handle_error_};
use crate::structure::Rational;

impl<'a> Document<'a> {
    pub fn camera_owner_name(&mut self) -> Option<String> {
        if let Some(s) = handle_error(self.lookup_string_raw(field::CameraOwnerName.into())) {
            Some(s)
        } else {
            handle_error(self.lookup_string_raw(field::CanonOwnerName.into()))
        }
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&mut self) -> Option<gufo_common::datetime::DateTime> {
        let datetime = handle_error(self.lookup_string(field::DateTimeOriginal.into()))?;
        let subsec = handle_error(self.lookup_string(field::SubSecTimeOriginal.into()));
        let offset = handle_error(self.lookup_string(field::OffsetTimeOriginal.into()));

        handle_error_(crate::structure::util::datetime(datetime, subsec, offset))
    }

    pub fn exposure_time(&mut self) -> Option<Rational<u32>> {
        handle_error(self.lookup_rational(field::ExposureTime.into()))
    }

    pub fn f_number(&mut self) -> Option<f32> {
        let f_number = handle_error(self.lookup_rational(field::FNumber.into()));

        f_number.map(|x| x.as_f32())
    }

    /// Focal length in mm
    pub fn focal_length(&mut self) -> Option<f32> {
        let focal_length = handle_error(self.lookup_rational(field::FocalLength.into()));

        focal_length.map(|x| x.as_f32())
    }

    pub fn gps_location(&mut self) -> Option<geography::Location> {
        let lat_ref = handle_error_(geography::LatRef::try_from(
            handle_error(self.lookup_string(field::GPSLatitudeRef.into()))?.as_str(),
        ))?;

        let [lat_ang, lat_min, lat_sec] =
            handle_error(self.lookup_rationals(field::GPSLatitude.into()))?;

        let lon_ref = handle_error_(geography::LonRef::try_from(
            handle_error(self.lookup_string(field::GPSLongitudeRef.into()))?.as_str(),
        ))?;

        let [lon_ang, lon_min, lon_sec] =
            handle_error(self.lookup_rationals(field::GPSLongitude.into()))?;

        Some(geography::Location::from_ref_coord(
            lat_ref,
            (lat_ang.as_f64(), lat_min.as_f64(), lat_sec.as_f64()),
            lon_ref,
            (lon_ang.as_f64(), lon_min.as_f64(), lon_sec.as_f64()),
        ))
    }

    pub fn iso_speed_rating(&mut self) -> Option<u16> {
        handle_error(self.lookup_short(field::PhotographicSensitivity.into()))
    }

    pub fn make(&mut self) -> Option<String> {
        handle_error(self.lookup_string_raw(field::Make.into()))
    }

    pub fn model(&mut self) -> Option<String> {
        handle_error(self.lookup_string_raw(field::Model.into()))
    }

    pub fn orientation(&mut self) -> Option<orientation::Orientation> {
        let orientation = handle_error(self.lookup_short(field::Orientation.into()))?;

        handle_error_(orientation::Orientation::try_from(orientation))
    }

    /// Software used
    pub fn software(&mut self) -> Option<String> {
        handle_error(self.lookup_string_raw(field::Software.into()))
    }

    pub fn user_comment(&mut self) -> Option<String> {
        let s =
            handle_error(self.lookup_character_identified_code_string(field::UserComment.into()))?;

        let tr = s.trim();
        if tr.is_empty() {
            None
        } else {
            Some(tr.to_string())
        }
    }
}
