//! A value type of an attribute.
//!
//! A union type to simplify the code.

/// A value type of an attribute.
pub enum Value {
    /// The length of a raw value.
    Raw(usize),
    /// A unsigned integer.
    U64(u64),
    /// A signed integer.
    I64(i64),
    /// A boolean flag.
    Bool(bool),
}

impl Value {
    /// Get as an u64. Returns None if this is not one.
    pub fn as_u64(&self) -> Option<u64> {
        let Self::U64(x) = self else { return None };
        Some(*x)
    }
    /// Get as an i64. Returns None if this is not one.
    pub fn as_i64(&self) -> Option<i64> {
        let Self::I64(x) = self else { return None };
        Some(*x)
    }
    /// Get the length of a Raw value. Returns None if this is not one.
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
