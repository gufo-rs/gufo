#[derive(Debug, PartialEq, Eq)]
pub struct AdditionOverflowError;
#[derive(Debug, PartialEq, Eq)]
pub struct SubstractionOverflowError;

pub trait U32Ext {
    fn usize(self) -> usize;
    fn i64(self) -> i64;
    fn safe_add(self, rhs: u32) -> Result<u32, AdditionOverflowError>;
    fn safe_sub(self, rhs: u32) -> Result<u32, SubstractionOverflowError>;
}

impl U32Ext for u32 {
    fn usize(self) -> usize {
        // Assume that systems are at least 32bit
        self.try_into().unwrap()
    }

    fn i64(self) -> i64 {
        self.into()
    }

    fn safe_add(self, rhs: u32) -> Result<u32, AdditionOverflowError> {
        self.checked_add(rhs).ok_or(AdditionOverflowError)
    }

    fn safe_sub(self, rhs: u32) -> Result<u32, SubstractionOverflowError> {
        self.checked_sub(rhs).ok_or(SubstractionOverflowError)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConversionOverflowError;

pub trait I64Ext {
    fn u32(self) -> Result<u32, ConversionOverflowError>;
    fn safe_add(self, rhs: i64) -> Result<i64, AdditionOverflowError>;
}

impl I64Ext for i64 {
    fn u32(self) -> Result<u32, ConversionOverflowError> {
        self.try_into().map_err(|_| ConversionOverflowError)
    }

    fn safe_add(self, rhs: i64) -> Result<i64, AdditionOverflowError> {
        self.checked_add(rhs).ok_or(AdditionOverflowError)
    }
}

pub fn apex_to_f_number(apex: f32) -> f32 {
    f32::sqrt(1.4).powf(apex)
}
