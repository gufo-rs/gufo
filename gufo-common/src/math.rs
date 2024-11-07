#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum MathError {
    #[error("Operation {0:?} + {1:?} failed")]
    AddFailed(Option<i128>, Option<i128>),
    #[error("Operation {0:?} - {1:?} failed")]
    SubFailed(Option<i128>, Option<i128>),
    #[error("Operation {0:?} * {1:?} failed")]
    MulFailed(Option<i128>, Option<i128>),
    #[error("Operation {0:?} / {1:?} failed")]
    DivFailed(Option<i128>, Option<i128>),
    #[error("Conversion failed for value {0:?}")]
    ConversionFailed(Option<i128>),
    #[error("Division Not Finite")]
    DivisionNotFinite,
    #[error("Negation overflowed")]
    NegationOverflow,
}

/// Container for safe integers operators
///
/// ```
/// # use gufo_common::math::Checked;
/// let x = Checked::new(2_u32);
/// let y = Checked::new(3_u32);
///
/// assert_eq!((x + y).unwrap(), 5);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Checked<T>(Result<T, MathError>);

impl<T> Checked<T> {
    pub fn new(val: T) -> Self {
        Self(Ok(val))
    }

    pub fn check(self) -> Result<T, MathError> {
        self.0
    }
}

impl<T> From<T> for Checked<T> {
    fn from(val: T) -> Self {
        Self(Ok(val))
    }
}

impl<T> std::ops::Deref for Checked<T> {
    type Target = Result<T, MathError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[macro_export]
/**
 * Rederines variables as [`Checked`].
 *
 * ```
 * use gufo_common::math::checked;
 *
 * let x = 1_u32;
 * let y = 2_u32;
 * checked![x];
 *
 * assert_eq!((x + y).unwrap(), 3);
 *
 * let x = 1_u32;
 * let y = 2_u32;
 * checked![y];
 *
 * assert_eq!((x + y).unwrap(), 3);
 *
 * let x = 5_u32;
 * let y = 2_u32;
 * checked![x, y,];
 *
 * assert_eq!((x * y).unwrap(), 10);
 *
 * let x = u32::MAX;
 * let y = 1;
 * checked![x, y,];
 *
 * assert!((x + y).is_err());
 * ```
 */
macro_rules! checked [
    ($($v:ident$(,)?)*) => {
        $( let $v = $crate::math::Checked::new($v); )*
    };
];

#[doc(hidden)]
#[macro_export]
macro_rules! mut_checked [
    ($v:ident) => {
        let mut $v = $crate::math::Checked::new($v);
    };
    ($($v:ident,)*) => {
        $( let mut $v = $crate::math::Checked::new($v); )*
    };
];

pub use {checked, mut_checked};

macro_rules! impl_operator {
    ($op:ident, $f:ident, $t:ty) => {
        paste::paste! {
            impl [< Safe $op >] for $t {
                fn [< safe_ $f >](self, rhs: $t) -> Result<$t, MathError> {
                    let err = || MathError:: [< $op Failed >] (self.try_into().ok(), rhs.try_into().ok());
                    self.[< checked_ $f >](rhs)
                        .ok_or_else(err)
                }
            }
        }

        impl<R: Into<Self> + Copy> std::ops::$op<R> for Checked<$t>
        {
            type Output = Self;

            #[inline]
            fn $f(self, rhs: R) -> Self::Output {
                let Checked(Ok(x)) = self else { return self };
                let Checked(Ok(y)) = rhs.into() else { return rhs.into() };
                paste::paste! {
                let res = x.[< safe_ $f >](y);
                }
                Checked(res)
            }
        }

        impl std::ops::$op<Checked<$t>> for $t
        {
            type Output = Checked<$t>;

            #[inline]
            fn $f(self, rhs: Checked<$t>) -> Self::Output {
                let y = match rhs.0 {
                    Err(err) => return Checked(Err(err)),
                    Ok(y) => y.try_into(),
                };
                let y: $t = match y {
                    Err(_) => return Checked(Err(MathError::ConversionFailed(Some(0)))),
                    Ok(y) => y,
                };
                paste::paste! {
                let res = self.[< safe_ $f >](y);
                }
                Checked(res)
            }
        }
    };
}

macro_rules! impl_binary_operators {
    ($t:ty) => {
        impl_operator!(Add, add, $t);
        impl_operator!(Sub, sub, $t);
        impl_operator!(Mul, mul, $t);
        impl_operator!(Div, div, $t);
    };
}

macro_rules! impl_cast {
    ($t:ty, $target:ident) => {
        impl Checked<$t> {
            pub fn $target(self) -> Checked<$target> {
                let x = match self.0 {
                    Err(err) => return Checked(Err(err)),
                    Ok(v) => v,
                };
                Checked(
                    x.try_into()
                        .map_err(|_| MathError::ConversionFailed(x.try_into().ok())),
                )
            }
        }
    };
}

macro_rules! impl_casts {
    ($t:ty) => {
        impl_cast!($t, u16);
        impl_cast!($t, u32);
        impl_cast!($t, u64);
        impl_cast!($t, i16);
        impl_cast!($t, i32);
        impl_cast!($t, i64);
        impl_cast!($t, usize);
    };
}

impl_binary_operators!(u16);
impl_binary_operators!(u32);
impl_binary_operators!(u64);
impl_binary_operators!(i16);
impl_binary_operators!(i32);
impl_binary_operators!(i64);
impl_binary_operators!(usize);

impl_casts!(u16);
impl_casts!(u32);
impl_casts!(u64);
impl_casts!(i16);
impl_casts!(i32);
impl_casts!(i64);
impl_casts!(usize);

pub trait ToU16: Sized + TryInto<u16> + TryInto<i128> + Copy {
    fn u16(self) -> Result<u16, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionFailed(self.try_into().ok()))
    }
}

impl ToU16 for i16 {}
impl ToU16 for i32 {}
impl ToU16 for u32 {}
impl ToU16 for i64 {}
impl ToU16 for u64 {}
impl ToU16 for usize {}

pub trait ToU32: Sized + TryInto<u32> + TryInto<i128> + Copy {
    fn u32(self) -> Result<u32, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionFailed(self.try_into().ok()))
    }
}

impl ToU32 for i16 {}
impl ToU32 for u16 {}
impl ToU32 for i32 {}
impl ToU32 for i64 {}
impl ToU32 for u64 {}
impl ToU32 for usize {}

pub trait ToI64: Sized + TryInto<i64> + TryInto<i128> + Copy {
    fn i64(self) -> Result<i64, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionFailed(self.try_into().ok()))
    }
}

impl ToI64 for i16 {}
impl ToI64 for u16 {}
impl ToI64 for i32 {}
impl ToI64 for u32 {}
impl ToI64 for u64 {}
impl ToI64 for usize {}

pub trait ToU64: Sized + TryInto<u64> + TryInto<i128> + Copy {
    fn u64(self) -> Result<u64, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionFailed(self.try_into().ok()))
    }
}

impl ToU64 for i16 {}
impl ToU64 for u16 {}
impl ToU64 for i32 {}
impl ToU64 for u32 {}
impl ToU64 for i64 {}
impl ToU64 for usize {}

pub trait ToUsize: Sized + TryInto<usize> + TryInto<i128> + Copy {
    fn usize(self) -> Result<usize, MathError> {
        self.try_into()
            .map_err(|_| MathError::ConversionFailed(self.try_into().ok()))
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

/// Same as `checked_sub` functions but returns an error
pub trait SafeSub: Sized {
    fn safe_sub(self, rhs: Self) -> Result<Self, MathError>;
}

/// Same as `checked_mul` functions but returns an error
pub trait SafeMul: Sized {
    fn safe_mul(self, rhs: Self) -> Result<Self, MathError>;
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
        self.checked_neg().ok_or(MathError::NegationOverflow)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MathError12 {
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
