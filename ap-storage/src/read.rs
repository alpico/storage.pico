//! Traits for reading.

use crate::{Offset, Error};
use core::mem::MaybeUninit;

/// Trait to read from a file or disk from a certain offset.
pub trait Read<'a> {
    /// Read into some byte buffer.
    async fn read_bytes(&'a mut self, offset: Offset, buf: &'a mut [u8]) -> Result<(), Error>;

}

/// Trait extension to simplify reading.
pub trait ReadExt<'a>: Read<'a> {
    /// Read a slice of sized objects.
    async fn read_slice<T: Sized>(&'a mut self, offset: Offset, buf: &'a mut [T]) -> Result<(), Error> {
        let x = unsafe {
            core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, core::mem::size_of_val(buf))
        };
        self.read_bytes(offset, x).await
    }

    /// Read a single object.
    async fn read_object<T: Sized>(&'a mut self, offset: Offset) -> Result<T, Error> {
        let mut res = MaybeUninit::uninit();
        let x = unsafe {
            core::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, core::mem::size_of::<T>())
        };
        self.read_bytes(offset, x).await?;
        Ok(unsafe { res.assume_init() })
    }
}
