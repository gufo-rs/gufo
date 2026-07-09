use std::fmt::Display;

use crate::{maybe_convertible_enum, types::Rational};

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
            x: PhysicalDimension::new(x_pixels as f64 / self.x.value(), self.x.unit()),
            y: PhysicalDimension::new(y_pixels as f64 / self.y.value(), self.y.unit()),
        }
    }

    pub fn dpi(&self) -> Self {
        self.convert(PhysicalDimensionUnit::Inch)
    }

    pub fn convert(&self, unit: PhysicalDimensionUnit) -> Self {
        Self {
            x: self.x.convert(unit),
            y: self.y.convert(unit),
        }
    }

    pub fn display(&self) -> Box<dyn Display> {
        let x_unit = match self.x.unit() {
            PhysicalDimensionUnit::Inch => String::from("DPI"),
            unit => format!("px/{}", unit.shorthand()),
        };

        if self.x == self.y {
            Box::new(format!("{}\u{2009}{}", self.x.value(), x_unit,))
        } else {
            let y_unit = match self.y.unit() {
                PhysicalDimensionUnit::Inch => String::from("DPI"),
                unit => format!("px/{}", unit.shorthand()),
            };

            Box::new(format!(
                "{}\u{2009}{} \u{d7} {}\u{2009}{}",
                self.x.value(),
                x_unit,
                self.y.value(),
                y_unit
            ))
        }
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

    pub fn dpi(&self) -> Self {
        self.convert(PhysicalDimensionUnit::Inch)
    }

    pub fn convert(&self, unit: PhysicalDimensionUnit) -> Self {
        Self {
            x: self.x.convert(unit),
            y: self.y.convert(unit),
        }
    }

    pub fn display(&self) -> Box<dyn Display> {
        Box::new(format!(
            "{}\u{2009}{} \u{d7} {}\u{2009}{}",
            self.x.value(),
            self.x.unit().shorthand(),
            self.y.value(),
            self.y.unit().shorthand(),
        ))
    }
}

maybe_convertible_enum!(
    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    #[cfg_attr(feature = "zvariant", derive(zvariant::Type))]
    #[cfg_attr(feature = "zvariant", zvariant(signature = "s"))]
    #[non_exhaustive]
    pub enum PhysicalDimensionUnit {
        Inch = 1,
        /// 1/6 inch
        Pica = 2,
        /// 1/72 inch
        Point = 3,
        Meter = 4,
        Centimeter = 5,
        Millimeter = 6,
    }
);

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
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// Return the value approximated as rational
    ///
    /// ```
    /// # use gufo_common::physical_dimension::*;
    /// # use gufo_common::types::Rational;
    /// let dim = PhysicalDimension::new(12.345, PhysicalDimensionUnit::Inch);
    /// assert_eq!(dim.value_rational(), Rational::new(12345, 1000));
    /// ```
    pub const fn value_rational(&self) -> Rational<u32> {
        const MAX_PRESISION: f64 = 1_000_000.;

        let x = (self.value() * MAX_PRESISION).round() / MAX_PRESISION;

        let mut div = 1.0_f64;
        while (x * div).fract() != 0. {
            div *= 10.;
        }

        Rational::new((x * div) as u32, div as u32)
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub const fn value_rational(&self) -> Rational<u32> {
        self.0.value_rational()
    }

    pub const fn unit(&self) -> PhysicalDimensionUnit {
        self.0.unit
    }

    pub const fn convert(self, unit: PhysicalDimensionUnit) -> Self {
        let value = self.0.value * unit.centimer_factor() / self.0.unit.centimer_factor();

        Self(PhysicalDimension::new(value, unit))
    }
}
