pub trait ToU16: Sized + TryInto<u16> {
    fn u16(self) -> Result<u16, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToU16 for i16 {}
impl ToU16 for i32 {}
impl ToU16 for u32 {}
impl ToU16 for i64 {}
impl ToU16 for u64 {}
impl ToU16 for usize {}

pub trait ToU32: Sized + TryInto<u32> {
    fn u32(self) -> Result<u32, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToU32 for i16 {}
impl ToU32 for u16 {}
impl ToU32 for i32 {}
impl ToU32 for i64 {}
impl ToU32 for u64 {}
impl ToU32 for usize {}

pub trait ToI64: Sized + TryInto<i64> {
    fn i64(self) -> Result<i64, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToI64 for i16 {}
impl ToI64 for u16 {}
impl ToI64 for i32 {}
impl ToI64 for u32 {}
impl ToI64 for u64 {}
impl ToI64 for usize {}

pub trait ToU64: Sized + TryInto<u64> {
    fn u64(self) -> Result<u64, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToU64 for i16 {}
impl ToU64 for u16 {}
impl ToU64 for i32 {}
impl ToU64 for u32 {}
impl ToU64 for i64 {}
impl ToU64 for usize {}

pub trait ToUsize: Sized + TryInto<usize> {
    fn usize(self) -> Result<usize, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionOverflowError)
    }
}

impl ToUsize for i16 {}
impl ToUsize for u16 {}
impl ToUsize for i32 {}
impl ToUsize for u32 {}
impl ToUsize for i64 {}
impl ToUsize for u64 {}

/// Same as `checked_add` functions but returns an error
pub trait SafeAdd: Sized {
    fn safe_add(self, rhs: Self) -> Result<Self, MathError>;
}

impl SafeAdd for u16 {
    fn safe_add(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_add(rhs)
            .ok_or(MathError::AdditionOverflowError)
    }
}

impl SafeAdd for u32 {
    fn safe_add(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_add(rhs)
            .ok_or(MathError::AdditionOverflowError)
    }
}

impl SafeAdd for i64 {
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

/// Same as `checked_sub` functions but returns an error
pub trait SafeSub: Sized {
    fn safe_sub(self, rhs: Self) -> Result<Self, MathError>;
}

impl SafeSub for u32 {
    fn safe_sub(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_sub(rhs)
            .ok_or(MathError::SubstractionOverflowError)
    }
}

impl SafeSub for i64 {
    fn safe_sub(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_sub(rhs)
            .ok_or(MathError::SubstractionOverflowError)
    }
}

/// Same as `checked_mul` functions but returns an error
pub trait SafeMul: Sized {
    fn safe_mul(self, rhs: Self) -> Result<Self, MathError>;
}

impl SafeMul for u32 {
    fn safe_mul(self, rhs: Self) -> Result<Self, MathError> {
        self.checked_mul(rhs)
            .ok_or(MathError::MultiplicationOverflowError)
    }
}

pub trait SafeDiv: Sized {
    fn safe_div(self, rhs: Self) -> Result<Self, MathError>;
}

impl SafeDiv for f64 {
    fn safe_div(self, rhs: Self) -> Result<Self, MathError> {
        let value = self / rhs;

        if value.is_infinite() {
            Err(MathError::DivisionNotFinite)
        } else {
            Ok(value)
        }
    }
}

/// Same as `checked_neg` functions but returns an error
pub trait SafeNeg: Sized {
    fn safe_neg(self) -> Result<Self, MathError>;
}

impl SafeNeg for i64 {
    fn safe_neg(self) -> Result<Self, MathError> {
        self.checked_neg().ok_or(MathError::NegationOverflowError)
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
    #[error("Multiplication overflowed")]
    MultiplicationOverflowError,
    #[error("Negation overflowed")]
    NegationOverflowError,
    #[error("Division gave non-finite float")]
    DivisionNotFinite,
}

/// Converts and APEX value to an F-Number
///
/// <https://en.wikipedia.org/wiki/APEX_system>
pub fn apex_to_f_number(apex: f32) -> f32 {
    f32::sqrt(1.4).powf(apex)
}
