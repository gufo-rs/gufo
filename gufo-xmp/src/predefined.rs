use gufo_common::field;

use super::Xmp;

impl Xmp {
    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.get_date_time(field::DateTimeOriginal)
    }

    pub fn model(&self) -> Option<String> {
        self.get(field::Model).map(ToString::to_string)
    }

    pub fn make(&self) -> Option<String> {
        self.get(field::Make).map(ToString::to_string)
    }

    pub fn f_number(&self) -> Option<f32> {
        if let Some(fnumer) = self.get_frac_f32(field::FNumber) {
            Some(fnumer)
        } else {
            let aperture_apex = self.get_frac_f32(field::Aperture)?;
            Some(gufo_common::math::apex_to_f_number(aperture_apex))
        }
    }

    pub fn exposure_time(&self) -> Option<(u32, u32)> {
        self.get_frac(field::ExposureTime)
    }

    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.get_u16(field::PhotographicSensitivity)
            .or_else(|| self.get_u16(field::ISOSpeedRatings))
    }

    pub fn focal_length(&self) -> Option<f32> {
        self.get_frac_f32(field::FocalLength)
    }

    pub fn creator(&self) -> Option<String> {
        self.get(field::Creator).map(ToString::to_string)
    }
}
