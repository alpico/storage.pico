//! Traits for writing.
use crate::{Offset, Error};


/// Trait to write to a file or disk at a certain offset.
pub trait Write<'a> {
    /// Write some byte buffer.
    async fn write_bytes(&'a mut self, offset: Offset, buf: &'a [u8]) -> Result<(), Error>;
}


/// Trait extension to simplify writing.
pub trait WriteExt<'a>: Write<'a> {
    /// Write a slice of sized objects.
    async fn write_slice<T: Sized>(&'a mut self, offset: Offset, buf: &'a [T]) -> Result<(), Error> {
        let x = unsafe {
            core::slice::from_raw_parts(buf.as_ptr() as *const u8, core::mem::size_of_val(buf))
        };
        self.write_bytes(offset, x).await
    }
}
