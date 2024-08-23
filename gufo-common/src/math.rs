pub trait ToU16: Sized + TryInto<u16> {
    fn u16(self) -> Result<u16, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToU16 for usize {}

pub trait ToI64: Sized + TryInto<i64> {
    fn i64(self) -> Result<i64, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToI64 for u16 {}

pub trait ToU64: Sized + TryInto<u64> {
    fn u64(self) -> Result<u64, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToU64 for usize {}

pub trait ToUsize: Sized + TryInto<usize> {
    fn usize(self) -> Result<usize, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToUsize for u64 {}

pub trait SafeAdd: Sized {
    fn safe_add(self, rhs: Self) -> Result<Self, MathError>;
}

impl SafeAdd for u16 {
    fn safe_add(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_add(rhs)
            .ok_or(MathError::AdditionOverflowError)
    }
}

impl SafeAdd for u64 {
    fn safe_add(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_add(rhs)
            .ok_or(MathError::AdditionOverflowError)
    }
}

impl SafeAdd for usize {
    fn safe_add(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_add(rhs)
            .ok_or(MathError::AdditionOverflowError)
    }
}

pub trait SafeSub: Sized {
    fn safe_sub(self, rhs: Self) -> Result<Self, MathError>;
}

impl SafeSub for i64 {
    fn safe_sub(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_sub(rhs)
            .ok_or(MathError::SubstractionOverflowError)
    }
}

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
