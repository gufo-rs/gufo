use std::fmt::Display;

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "zvariant",
    derive(zvariant::DeserializeDict, zvariant::SerializeDict, zvariant::Type)
)]
#[cfg_attr(feature = "zvariant", zvariant(signature = "dict"))]
pub struct PixelDensity {
    x: PixelsPerPhysicalDimension,
    y: PixelsPerPhysicalDimension,
}

impl PixelDensity {
    pub fn new(x: PixelsPerPhysicalDimension, y: PixelsPerPhysicalDimension) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> PixelsPerPhysicalDimension {
        self.x
    }

    pub fn y(&self) -> PixelsPerPhysicalDimension {
        self.y
    }

    pub fn physical_size(self, x_pixels: u32, y_pixels: u32) -> PhysicalSize {
        PhysicalSize {
            x: PhysicalDimension::new(self.x.value() * x_pixels as f64, self.x.unit()),
            y: PhysicalDimension::new(self.y.value() * y_pixels as f64, self.y.unit()),
        }
    }

    pub fn display(&self) -> Box<dyn Display> {
        Box::new(format!(
            "{}\u{2009}px/{} \u{d7} {}\u{2009}px/{}",
            self.x.value(),
            self.x.unit().shorthand(),
            self.y.value(),
            self.y.unit().shorthand()
        ))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "zvariant",
    derive(zvariant::DeserializeDict, zvariant::SerializeDict, zvariant::Type)
)]
#[cfg_attr(feature = "zvariant", zvariant(signature = "dict"))]
#[non_exhaustive]
pub struct PhysicalSize {
    pub x: PhysicalDimension,
    pub y: PhysicalDimension,
}

impl PhysicalSize {
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
    value: f64,
    unit: PhysicalDimensionUnit,
}

impl PhysicalDimension {
    pub const fn new(value: f64, unit: PhysicalDimensionUnit) -> Self {
        Self { value, unit }
    }

    pub const fn value(&self) -> f64 {
        self.value
    }

    pub const fn unit(&self) -> PhysicalDimensionUnit {
        self.unit
    }

    /// Convert to different physical dimension
    ///
    /// ```
    /// # use gufo_common::physical_dimension::{PhysicalDimension, PhysicalDimensionUnit};
    /// assert_eq!(
    ///     PhysicalDimension::new(1., PhysicalDimensionUnit::Inch)
    ///         .convert(PhysicalDimensionUnit::Centimeter)
    ///         .value(),
    ///     2.54
    /// );
    /// assert_eq!(
    ///     PhysicalDimension::new(2., PhysicalDimensionUnit::Meter)
    ///         .convert(PhysicalDimensionUnit::Centimeter)
    ///         .value(),
    ///     200.
    /// );
    /// ```
    pub const fn convert(self, unit: PhysicalDimensionUnit) -> Self {
        let value = self.value * self.unit.centimer_factor() / unit.centimer_factor();

        Self { unit, value }
    }
}

#[cfg_attr(
    feature = "zvariant",
    derive(serde::Deserialize, serde::Serialize, zvariant::Type)
)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "zvariant", zvariant(signature = "dict"))]
#[repr(transparent)]
pub struct PixelsPerPhysicalDimension(PhysicalDimension);

impl PixelsPerPhysicalDimension {
    pub const fn new(value: f64, unit: PhysicalDimensionUnit) -> Self {
        Self(PhysicalDimension::new(value, unit))
    }

    pub const fn value(&self) -> f64 {
        self.0.value
    }

    pub const fn unit(&self) -> PhysicalDimensionUnit {
        self.0.unit
    }

    pub const fn convert(self, unit: PhysicalDimensionUnit) -> Self {
        let value = self.0.value * unit.centimer_factor() / self.0.unit.centimer_factor();

        Self(PhysicalDimension::new(value, unit))
    }
}
