use std::cell::RefCell;

use gufo_common::exif::Field;
use gufo_common::field::{self};
use gufo_common::orientation;

use crate::error::Result;
use crate::internal::*;

#[derive(Debug)]
pub struct Exif {
    decoder: RefCell<ExifRaw>,
}

impl Exif {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut decoder = ExifRaw::new(data);
        decoder.decode()?;

        Ok(Self {
            decoder: RefCell::new(decoder),
        })
    }

    /// Image orientation
    ///
    /// Rotation and mirroring that have to be applied to show the image
    /// correctly
    pub fn orientation(&self) -> orientation::Orientation {
        self.decoder
            .borrow_mut()
            .lookup_short(TagIfd::new(
                field::Orientation::TAG,
                field::Orientation::IFD,
            ))
            .ok()
            .flatten()
            .and_then(|x| orientation::Orientation::try_from(x).ok())
            .unwrap_or(orientation::Orientation::Id)
    }

    /// Camera manifacturer
    pub fn make(&self) -> Option<String> {
        self.decoder.borrow_mut().lookup_string(field::Make).ok()?
    }

    /// Camera model
    pub fn model(&self) -> Option<String> {
        self.decoder
            .borrow_mut()
            .lookup_string(TagIfd::new(field::Model::TAG, field::Model::IFD))
            .ok()?
    }

    /// ISO
    pub fn iso_speed_rating(&self) -> Option<u16> {
        self.decoder
            .borrow_mut()
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
            .borrow_mut()
            .lookup_rational(TagIfd::new(field::FNumber::TAG, field::FNumber::IFD))
            .ok()??;

        Some(x as f32 / y as f32)
    }

    /// Focal length in mm
    pub fn focal_length(&self) -> Option<f32> {
        let (x, y) = self
            .decoder
            .borrow_mut()
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
            .borrow_mut()
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
            .borrow_mut()
            .lookup_datetime(field::DateTimeOriginal)
            .ok()??;

        // Add sub-seconds
        if let Some(subsec) = self
            .decoder
            .borrow_mut()
            .lookup_string(field::SubSecTimeOriginal)
            .ok()
            .flatten()
        {
            // Remove NULL as well since iPhone 15 and HTC ONE have a leading NULL in this
            // field
            let subsec = subsec.trim().replace('\0', "");
            if !subsec.is_empty() {
                datetime.push('.');
                datetime.push_str(&subsec);
            }
        }

        let use_offset;

        // Add offset (timezone)
        if let Some(offset) = self
            .decoder
            .borrow_mut()
            .lookup_string(field::OffsetTimeOriginal)
            .ok()
            .flatten()
        {
            datetime.push_str(&offset);
            use_offset = true;
        } else {
            // Add a offset to allow parser to work
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

    pub fn debug_dump(&self) -> String {
        self.decoder.borrow_mut().debug_dump()
    }
}
