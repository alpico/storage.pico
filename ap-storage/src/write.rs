//! Traits for writing.
use crate::{Error, Offset};

/// Write to a file or disk at a certain offset.
pub trait Write<'a> {
    /// Write some byte buffer.
    fn write_bytes(&'a self, offset: Offset, buf: &'a [u8]) -> Result<usize, Error>;
}

/// Trait extension to simplify writing.
pub trait WriteExt<'a>: Write<'a> {
    /// Write a slice of sized objects.
    fn write_slice<T: Sized>(&'a self, offset: Offset, buf: &'a [T]) -> Result<usize, Error> {
        let x = unsafe {
            core::slice::from_raw_parts(buf.as_ptr() as *const u8, core::mem::size_of_val(buf))
        };
        let res = self.write_bytes(offset, x)?;
        Ok(res as usize / core::mem::size_of::<T>())
    }
}
