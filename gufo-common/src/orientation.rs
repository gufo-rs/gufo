use std::ops::{Add, Sub};

crate::utils::maybe_convertible_enum!(
    #[repr(u16)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Operations that have to be applied to orient the image correctly
    pub enum Orientation {
        Id = 1,
        Rotation90 = 8,
        Rotation180 = 3,
        Rotation270 = 6,
        Mirrored = 2,
        MirroredRotation90 = 5,
        MirroredRotation180 = 4,
        MirroredRotation270 = 7,
    }
);

impl Orientation {
    pub fn new(rotation: Rotation, mirrored: bool) -> Self {
        match (mirrored, rotation) {
            (false, Rotation::_0) => Self::Id,
            (false, Rotation::_90) => Self::Rotation90,
            (false, Rotation::_180) => Self::Rotation180,
            (false, Rotation::_270) => Self::Rotation270,
            (true, Rotation::_0) => Self::Mirrored,
            (true, Rotation::_90) => Self::MirroredRotation90,
            (true, Rotation::_180) => Self::MirroredRotation180,
            (true, Rotation::_270) => Self::MirroredRotation270,
        }
    }
}

#[derive(Debug)]
pub struct UnknownOrientation;

#[allow(dead_code)]
#[derive(Debug)]
/// Rotation was not given in multiples of 90
pub struct InvalidRotation(f64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Rotation {
    _0,
    _90,
    _180,
    _270,
}

impl Rotation {
    pub fn degrees(self) -> u16 {
        match self {
            Rotation::_0 => 0,
            Rotation::_90 => 90,
            Rotation::_180 => 180,
            Rotation::_270 => 270,
        }
    }
}

impl Add for Rotation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Rotation::try_from((self.degrees().checked_add(rhs.degrees()).unwrap()) as f64).unwrap()
    }
}

impl Sub for Rotation {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Rotation::try_from((self.degrees() as f64) - (rhs.degrees() as f64)).unwrap()
    }
}

/// Get rotation from multiples of 90 deg
///
/// The given value is rounded to an integer number
///
/// ```
/// # use gufo_common::orientation::Rotation;
/// assert_eq!(Rotation::try_from(90.).unwrap(), Rotation::_90);
/// assert_eq!(Rotation::try_from(-90.).unwrap(), Rotation::_270);
/// assert!(Rotation::try_from(1.).is_err());
/// ```
impl TryFrom<f64> for Rotation {
    type Error = InvalidRotation;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let rotation = value.round().rem_euclid(360.);

        match rotation {
            0. => Ok(Self::_0),
            90. => Ok(Self::_90),
            180. => Ok(Self::_180),
            270. => Ok(Self::_270),
            _ => Err(InvalidRotation(value)),
        }
    }
}

impl Orientation {
    pub fn mirror(self) -> bool {
        matches!(
            self,
            Self::Mirrored
                | Self::MirroredRotation90
                | Self::MirroredRotation180
                | Self::MirroredRotation270
        )
    }

    pub fn rotate(self) -> Rotation {
        match self {
            Self::Id | Self::Mirrored => Rotation::_0,
            Self::Rotation90 | Self::MirroredRotation90 => Rotation::_90,
            Self::Rotation180 | Self::MirroredRotation180 => Rotation::_180,
            Self::Rotation270 | Self::MirroredRotation270 => Rotation::_270,
        }
    }
}
