//! Support for extended attributes.

/// The type of the attribute gives a hint how they can be fetched.
pub enum AttrType {
    Raw,
    U64,
    I64,
}

/// Trait for file attributes aka (key,value) pairs.
///
/// All keys can be iterated.
pub trait Attributes<'a>: IntoIterator<Item = &'a (AttrType, &'a str)> {
    /// Get a binary encoded attribute.
    ///
    /// Returns the length of the value to be written if value is large enough.
    fn get_raw(&mut self, _name: &str, _value: &mut [u8]) -> Option<usize> {
        None
    }

    /// Get a u64 attribute.
    fn get_u64(&mut self, _name: &str) -> Option<u64> {
        None
    }

    /// Get a i64 attribute.
    fn get_i64(&mut self, _name: &str) -> Option<i64> {
        None
    }
}
