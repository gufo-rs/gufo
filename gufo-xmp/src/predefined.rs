use gufo_common::types::Rational;
use gufo_common::{field, orientation};

use super::Xmp;

impl Xmp {
    pub fn creator(&self) -> Option<String> {
        self.get(field::Creator).map(ToString::to_string)
    }

    pub fn creator_tool(&self) -> Option<String> {
        self.get(field::CreatorTool).map(ToString::to_string)
    }

    pub fn camera_owner_name(&self) -> Option<String> {
        self.get(field::CameraOwnerName).map(ToString::to_string)
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        self.get_date_time(field::DateTimeOriginal)
    }

    pub fn digital_zoom_ratio(&self) -> Option<Rational<u32>> {
        self.get_frac(field::DigitalZoomRatio)
    }

    pub fn exposure_time(&self) -> Option<Rational<u32>> {
        self.get_frac(field::ExposureTime)
    }

    pub fn f_number(&self) -> Option<f32> {
        if let Some(fnumer) = self.get_frac(field::FNumber) {
            Some(fnumer.as_f32())
        } else {
            let aperture_apex = self.get_frac_f32(field::Aperture)?;
            Some(gufo_common::math::apex_to_f_number(aperture_apex))
        }
    }

    pub fn focal_length(&self) -> Option<Rational<u32>> {
        self.get_frac(field::FocalLength)
    }

    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.get_u16(field::PhotographicSensitivity)
            .or_else(|| self.get_u16(field::ISOSpeedRatings))
    }

    pub fn lens_make(&self) -> Option<String> {
        self.get(field::LensMake).map(ToString::to_string)
    }

    pub fn lens_model(&self) -> Option<String> {
        self.get(field::LensMake).map(ToString::to_string)
    }

    pub fn make(&self) -> Option<String> {
        self.get(field::Make).map(ToString::to_string)
    }

    pub fn model(&self) -> Option<String> {
        self.get(field::Model).map(ToString::to_string)
    }

    pub fn orientation(&self) -> Option<orientation::Orientation> {
        orientation::Orientation::try_from(
            self.get(field::UserComment)
                .and_then(|x| str::parse::<u16>(x).ok())?,
        )
        .ok()
    }

    pub fn rights(&self) -> Option<String> {
        self.get(field::Rights).map(ToString::to_string)
    }

    pub fn rights_web_statement(&self) -> Option<String> {
        self.get(field::WebStatement).map(ToString::to_string)
    }

    pub fn software(&self) -> Option<String> {
        self.get(field::CreatorTool).map(ToString::to_string)
    }

    pub fn user_comment(&self) -> Option<String> {
        self.get(field::UserComment).map(ToString::to_string)
    }
}
