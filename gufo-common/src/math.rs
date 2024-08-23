pub trait U32Ext {
    fn usize(self) -> usize;
    fn i64(self) -> i64;
    fn safe_add(self, rhs: u32) -> Result<u32, MathError>;
    fn safe_sub(self, rhs: u32) -> Result<u32, MathError>;
}

impl U32Ext for u32 {
    fn usize(self) -> usize {
        // Assume that systems are at least 32bit
        self.try_into().unwrap()
    }

    fn i64(self) -> i64 {
        self.into()
    }

    fn safe_add(self, rhs: u32) -> Result<u32, MathError> {
        self.checked_add(rhs)
            .ok_or(MathError::AdditionOverflowError)
    }

    fn safe_sub(self, rhs: u32) -> Result<u32, MathError> {
        self.checked_sub(rhs)
            .ok_or(MathError::SubstractionOverflowError)
    }
}

pub trait I64Ext {
    fn u32(self) -> Result<u32, MathError>;
    fn safe_add(self, rhs: i64) -> Result<i64, MathError>;
}

impl I64Ext for i64 {
    fn u32(self) -> Result<u32, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }

    fn safe_add(self, rhs: i64) -> Result<i64, MathError> {
        self.checked_add(rhs)
            .ok_or(MathError::AdditionOverflowError)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MathError {
    #[error("Addition overflowed")]
    AdditionOverflowError,
    #[error("Type conversion overflowed")]
    ConversionOverflowError,
    #[error("Substraction overflowed")]
    SubstractionOverflowError,
}

pub fn apex_to_f_number(apex: f32) -> f32 {
    f32::sqrt(1.4).powf(apex)
}
