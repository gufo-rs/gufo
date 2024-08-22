#[derive(Debug, Clone)]
pub enum DateTime {
    FixedOffset(chrono::DateTime<chrono::FixedOffset>),
    Naive(chrono::NaiveDateTime),
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FixedOffset(d) => write!(f, "{}", d),
            Self::Naive(d) => write!(f, "{}", d),
        }
    }
}
