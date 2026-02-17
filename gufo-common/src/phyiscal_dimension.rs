#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "zvariant",
    derive(zvariant::DeserializeDict, zvariant::SerializeDict, zvariant::Type)
)]
#[cfg_attr(feature = "zvariant", zvariant(signature = "dict"))]
#[non_exhaustive]
pub struct PhysicalDimensions {
    pub x: PhysicalDimension,
    pub y: PhysicalDimension,
}

impl PhysicalDimensions {
    pub fn new(x: PhysicalDimension, y: PhysicalDimension) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "zvariant", derive(zvariant::Type))]
#[non_exhaustive]
pub enum PhysicalDimensionUnit {
    Inch,
    /// 1/6 inch
    Pica,
    /// 1/72 inch
    Point,
    Meter,
    Centimeter,
    Millimeter,
}

impl PhysicalDimensionUnit {
    pub const fn centimer_factor(self) -> f64 {
        match self {
            Self::Inch => 2.54,
            Self::Pica => 2.54 / 6.,
            Self::Point => 2.54 / 72.,
            Self::Meter => 100.,
            Self::Centimeter => 1.,
            Self::Millimeter => 1. / 10.,
        }
    }

    pub const fn shorthand(self) -> &'static str {
        match self {
            Self::Inch => "in",
            Self::Pica => "pc",
            Self::Point => "pt",
            Self::Meter => "m",
            Self::Centimeter => "cm",
            Self::Millimeter => "mm",
        }
    }
}

#[cfg_attr(
    feature = "zvariant",
    derive(zvariant::DeserializeDict, zvariant::SerializeDict, zvariant::Type)
)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "zvariant", zvariant(signature = "dict"))]
#[non_exhaustive]
pub struct PhysicalDimension {
    pub value: f64,
    pub unit: PhysicalDimensionUnit,
}

impl PhysicalDimension {
    pub const fn new(value: f64, unit: PhysicalDimensionUnit) -> Self {
        Self { value, unit }
    }

    /// Convert to different physical dimension
    ///
    /// ```
    /// # use glycin_common::{PhysicalDimension, PhysicalDimensionUnit};
    /// assert_eq!(PhysicalDimension::new(1., PhysicalDimensionUnit::Inch).convert(PhysicalDimensionUnit::Centimeter).value, 2.54);
    /// assert_eq!(PhysicalDimension::new(2., PhysicalDimensionUnit::Meter).convert(PhysicalDimensionUnit::Centimeter).value, 200.);
    /// ```
    pub const fn convert(self, unit: PhysicalDimensionUnit) -> Self {
        let value = self.value * self.unit.centimer_factor() / unit.centimer_factor();

        Self { unit, value }
    }
}
