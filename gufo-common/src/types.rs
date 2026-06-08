use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational<T> {
    pub numerator: T,
    pub denominator: T,
}

impl<T> Rational<T> {
    pub fn new(numerator: T, denominator: T) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl Rational<u32> {
    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }

    pub fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

impl Rational<i32> {
    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }
}

impl<T: Display> Rational<T> {
    pub fn display(&self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }
}
