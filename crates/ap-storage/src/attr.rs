//! Support for extended attributes.
//!
//! # Naming
//!
//! The attribute name-space is global.  To avoid conflicts we prefix
//! the name with the crate it is defined in.
//!
//! # Types
//!
//! - there are signed and unsigned 64-bit values
//! - there a
//! - times are defined as nano-seconds since UNIX_EPOCH.

/// Define an attribute.
#[macro_export]
macro_rules! new_attr {
    ($name: ident, $typ:ident, $doc:literal) => {
        #[doc=concat!($doc, " (", stringify!($typ), ")")]
        pub const $name: &str = concat!(env!("CARGO_PKG_NAME"), ".", stringify!($name));
    };
}
new_attr!(ATIME, I64, "Time of last file access.");
new_attr!(BTIME, I64, "Time of file birth.");
new_attr!(FTYPE, Raw, "File type.");
new_attr!(ID, U64, "A unique ID of the file, used to detect hard-links.");
new_attr!(MTIME, I64, "Time of last file modification.");
new_attr!(SIZE, U64, "The size of the file in bytes.");

/// A typed value of an attribute.
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

/// Trait for file attributes aka (key,value) pairs.
///
/// All keys can be iterated.
pub trait Attributes<'a>: IntoIterator<Item = &'a &'a str> {
    /// Get a value.  The (optional) buffer is used when raw-values are returned.
    fn get(&self, name: &str, buf: &mut [u8]) -> Option<Value>;
}
