//! Support for extended attributes.

use super::Error;

pub struct Entry {
    /// Length of the name in bytes.
    pub name_len: usize,
    /// Length of the value in bytes.
    pub value_len: usize,
}

/// Trait for iterating over extended attributes aka (key,value) pairs.
pub trait Attributes {
    /// Return the next attribute.
    ///
    /// - Data is copied out to the buffers and truncated if they are to small.
    /// - An `Ok(None)` means end of iterator.
    fn next(&mut self, name: &mut [u8], value: &mut [u8]) -> Result<Option<Entry>, Error>;
}

/// Helper for filesystems without attributes.
pub struct EmptyAttributes;
impl Attributes for EmptyAttributes {
    fn next(&mut self, _name: &mut [u8], _value: &mut [u8]) -> Result<Option<Entry>, Error> {
        Ok(None)
    }
}
