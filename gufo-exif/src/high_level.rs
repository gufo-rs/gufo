use std::sync::Mutex;

use gufo_common::exif::Field;
use gufo_common::field::{self};
use gufo_common::{geography, orientation};

use crate::error::Result;
use crate::internal::*;

#[derive(Debug)]
pub struct Exif {
    decoder: Mutex<ExifRaw>,
}

impl Exif {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut decoder = ExifRaw::new(data);
        decoder.decode()?;

        Ok(Self {
            decoder: Mutex::new(decoder),
        })
    }

    /// Image orientation
    ///
    /// Rotation and mirroring that have to be applied to show the image
    /// correctly
    pub fn orientation(&self) -> Option<orientation::Orientation> {
        self.decoder
            .lock()
            .unwrap()
            .lookup_short(TagIfd::new(
                field::Orientation::TAG,
                field::Orientation::IFD,
            ))
            .ok()
            .flatten()
            .and_then(|x| orientation::Orientation::try_from(x).ok())
    }

    pub fn gps_location(&self) -> Option<geography::Location> {
        let lat_ref = geography::LatRef::try_from(
            self.decoder
                .lock()
                .unwrap()
                .lookup_string(field::GPSLatitudeRef)
                .ok()
                .flatten()?
                .as_str(),
        )
        .ok()?;

        let [lat_ang, lat_min, lat_sec] = self
            .decoder
            .lock()
            .unwrap()
            .lookup_rationals_f64(field::GPSLatitude)
            .ok()
            .flatten()?;

        let lon_ref = geography::LonRef::try_from(
            self.decoder
                .lock()
                .unwrap()
                .lookup_string(field::GPSLongitudeRef)
                .ok()
                .flatten()?
                .as_str(),
        )
        .ok()?;

        let [lon_ang, lon_min, lon_sec] = self
            .decoder
            .lock()
            .unwrap()
            .lookup_rationals_f64(field::GPSLongitude)
            .ok()
            .flatten()?;

        Some(geography::Location::from_ref_coord(
            lat_ref,
            (lat_ang, lat_min, lat_sec),
            lon_ref,
            (lon_ang, lon_min, lon_sec),
        ))
    }

    /// Camera manifacturer
    pub fn make(&self) -> Option<String> {
        self.decoder
            .lock()
            .unwrap()
            .lookup_string(field::Make)
            .ok()?
    }

    /// Camera model
    pub fn model(&self) -> Option<String> {
        self.decoder
            .lock()
            .unwrap()
            .lookup_string(TagIfd::new(field::Model::TAG, field::Model::IFD))
            .ok()?
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.decoder
            .lock()
            .unwrap()
            .lookup_short(TagIfd::new(
                field::PhotographicSensitivity::TAG,
                field::PhotographicSensitivity::IFD,
            ))
            .ok()?
    }

    /// Aperture
    pub fn f_number(&self) -> Option<f32> {
        let (x, y) = self
            .decoder
            .lock()
            .unwrap()
            .lookup_rational(TagIfd::new(field::FNumber::TAG, field::FNumber::IFD))
            .ok()??;

        Some(x as f32 / y as f32)
    }

    /// Focal length in mm
    pub fn focal_length(&self) -> Option<f32> {
        let (x, y) = self
            .decoder
            .lock()
            .unwrap()
            .lookup_rational(TagIfd::new(
                field::FocalLength::TAG,
                field::FocalLength::IFD,
            ))
            .ok()??;

        Some(x as f32 / y as f32)
    }

    /// Exposure time in seconds
    ///
    /// Fraction of first element devided by second element. The first element
    /// is typically one, such that the value is given in its common for like
    /// "1/60 sec".
    pub fn exposure_time(&self) -> Option<(u32, u32)> {
        self.decoder
            .lock()
            .unwrap()
            .lookup_rational(TagIfd::new(
                field::ExposureTime::TAG,
                field::ExposureTime::IFD,
            ))
            .ok()?
    }

    #[cfg(feature = "chrono")]
    pub fn date_time_original(&self) -> Option<gufo_common::datetime::DateTime> {
        let mut datetime = self
            .decoder
            .lock()
            .unwrap()
            .lookup_datetime(field::DateTimeOriginal)
            .ok()??;

        // Add sub-seconds
        if let Some(subsec) = self
            .decoder
            .lock()
            .unwrap()
            .lookup_string(field::SubSecTimeOriginal)
            .ok()
            .flatten()
        {
            // Remove NULL as well since iPhone 15 and HTC ONE have a leading NULL in this
            // field
            let subsec = subsec.trim();
            if !subsec.is_empty() {
                datetime.push('.');
                datetime.push_str(&subsec);
            }
        }

        let use_offset;

        // Add offset (timezone)
        if let Some(offset) = self
            .decoder
            .lock()
            .unwrap()
            .lookup_string(field::OffsetTimeOriginal)
            .ok()
            .flatten()
        {
            datetime.push_str(&offset);
            use_offset = true;
        } else {
            // Add an offset to allow parser to work
            datetime.push('Z');
            use_offset = false;
        }

        let x = chrono::DateTime::parse_from_rfc3339(&datetime).ok()?;

        Some(if use_offset {
            gufo_common::datetime::DateTime::FixedOffset(x)
        } else {
            gufo_common::datetime::DateTime::Naive(x.naive_utc())
        })
    }

    pub fn camera_owner(&self) -> Option<String> {
        let mut decoder = self.decoder.lock().unwrap();

        if let Some(s) = decoder.lookup_string(field::CameraOwnerName).ok().flatten() {
            Some(s)
        } else if let Some(s) = decoder
            .lookup_string_raw(field::CanonOwnerName)
            .ok()
            .flatten()
        {
            let bytes = s.into_iter().take_while(|x| *x != 0).collect::<Vec<_>>();
            let s = String::from_utf8_lossy(&bytes);
            Some(s.to_string())
        } else {
            None
        }
    }

    /// Software used
    pub fn software(&self) -> Option<String> {
        self.decoder
            .lock()
            .unwrap()
            .lookup_string(field::Software)
            .ok()?
    }

    pub fn user_comment(&self) -> Option<String> {
        let s = self
            .decoder
            .lock()
            .unwrap()
            .lookup_character_identified_code_string(field::UserComment)
            .ok()
            .flatten()?;

        let tr = s.trim();
        if tr.is_empty() {
            None
        } else {
            Some(tr.to_string())
        }
    }

    pub fn decoder(&mut self) -> std::sync::MutexGuard<ExifRaw> {
        self.decoder.lock().unwrap()
    }

    pub fn debug_dump(&self) -> String {
        self.decoder.lock().unwrap().debug_dump()
    }
}
