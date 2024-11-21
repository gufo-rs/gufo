//! Coding-independent code points
//!
//! - [ITU-T H.273: Coding-independent code points for video signal type identification](https://www.itu.int/rec/T-REC-H.273)

use crate::utils;

/// Coding-independent code point
pub struct Cicp {
    pub colour_primaries: ColourPrimaries,
    pub transfer_characteristics: TransferCharacteristics,
    pub matrix_coefficients: MatrixCoefficients,
    pub video_full_range_flag: VideoRangeFlag,
}

impl Cicp {
    /// Get CICP from bytes in the order of struct definition
    ///
    /// ```
    /// # use gufo_common::cicp::*;
    /// let cicp = Cicp::from_bytes(&[0x09, 0x10, 0x00, 0x01]).unwrap();
    ///
    /// assert_eq!(cicp.colour_primaries, ColourPrimaries::Rec2020);
    /// assert_eq!(cicp.transfer_characteristics, TransferCharacteristics::Pq);
    /// assert_eq!(cicp.matrix_coefficients, MatrixCoefficients::Identity);
    /// assert_eq!(cicp.video_full_range_flag, VideoRangeFlag::Full);
    /// ```
    pub fn from_bytes(bytes: &[u8; 4]) -> Result<Self, CicpError> {
        let colour_primaries = ColourPrimaries::from(bytes[0]);
        let transfer_characteristics = TransferCharacteristics::from(bytes[1]);
        let matrix_coefficients: MatrixCoefficients = MatrixCoefficients::from(bytes[2]);
        let video_full_range_flag = VideoRangeFlag::try_from(bytes[3])
            .map_err(|err| CicpError::InvalidVideoFullRangeFlag(err.0))?;

        Ok(Self {
            colour_primaries,
            transfer_characteristics,
            matrix_coefficients,
            video_full_range_flag,
        })
    }
}

utils::convertible_enum!(
    #[repr(u8)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum ColourPrimaries {
        Srgb = 1,
        Unspecified = 2,
        Rec2020 = 9,
        DciP3 = 12,
    }
);

utils::convertible_enum!(
    #[repr(u8)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum TransferCharacteristics {
        /// Standard dynamic range
        Sdr = 1,
        Unspecified = 2,
        /// Standard dynamic range 10 bit
        Sdr_ = 6,
        /// Standard dynamic range 10 bit
        Sdr10 = 14,
        /// Standard dynamic range 12 bit
        Sdr12 = 15,
        /// Perceptual quantization (PQ) system
        Pq = 16,
        /// Hybrid log-gamma (HLG) system
        Hlg = 18,
    }
);

utils::convertible_enum!(
    #[repr(u8)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum MatrixCoefficients {
        Identity = 0,
        Unspecified = 2,
        ICtCp = 14,
    }
);

utils::maybe_convertible_enum!(
    #[repr(u8)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum VideoRangeFlag {
        Narrow = 0,
        Full = 1,
    }
);

#[derive(Debug, thiserror::Error)]
pub enum CicpError {
    #[error("Invalid video full range flag '{0}'. Expected '0' or '1'.")]
    InvalidVideoFullRangeFlag(u8),
}
