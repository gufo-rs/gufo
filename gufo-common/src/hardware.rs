use crate::types::Rational;

#[derive(Debug)]
pub struct LensSpecification {
    /// Minimal folcal length in mm
    pub min_focal_length: Rational<u32>,
    /// Maximal folcal length in mm
    pub max_focal_length: Rational<u32>,
    pub min_f_number_min_focal_length: Rational<u32>,
    pub min_f_number_max_focal_length: Rational<u32>,
}

impl LensSpecification {
    pub fn display(&self) -> String {
        let mut s = if self.min_focal_length == self.max_focal_length {
            format!("{}\u{202F}mm", self.min_focal_length.as_f32())
        } else {
            format!(
                "{}–{}\u{202F}mm",
                self.min_focal_length.as_f32(),
                self.max_focal_length.as_f32()
            )
        };

        if self.min_f_number_max_focal_length.numerator > 0
            && self.min_f_number_max_focal_length.denominator > 0
        {
            if self.min_f_number_min_focal_length == self.min_f_number_max_focal_length {
                s.push_str(&format!(
                    " \u{192}\u{2215}{}",
                    self.min_f_number_min_focal_length.as_f32()
                ));
            } else {
                s.push_str(&format!(
                    " \u{192}\u{2215}{}–{}",
                    self.min_f_number_min_focal_length.as_f32(),
                    self.min_f_number_max_focal_length.as_f32()
                ));
            }
        }

        s
    }
}
