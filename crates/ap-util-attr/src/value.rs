//! A value type of an attribute.
//!
//! A union type to simplify the code.

/// A value type of an attribute.
pub enum Value {
    Raw(usize),
    U64(u64),
    I64(i64),
    Bool(bool),
}

impl Value {
    pub fn as_u64(&self) -> Option<u64> {
        let Self::U64(x) = self else { return None };
        Some(*x)
    }
    pub fn as_i64(&self) -> Option<i64> {
        let Self::I64(x) = self else { return None };
        Some(*x)
    }
    pub fn as_len(&self) -> Option<usize> {
        let Self::Raw(x) = self else { return None };
        Some(*x)
    }
}
impl From<u8> for Value {
    fn from(val: u8) -> Self {
        Value::U64(val as u64)
    }
}
impl From<u16> for Value {
    fn from(val: u16) -> Self {
        Value::U64(val as u64)
    }
}
impl From<u32> for Value {
    fn from(val: u32) -> Self {
        Value::U64(val as u64)
    }
}
impl From<u64> for Value {
    fn from(val: u64) -> Self {
        Value::U64(val)
    }
}
impl From<i64> for Value {
    fn from(val: i64) -> Self {
        Value::I64(val)
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Bool(val)
    }
}
