//! Traits for reading.

use crate::{Offset, Error};
use core::mem::MaybeUninit;

/// Trait to read from a file or disk from a certain offset.
pub trait Read {
    /// Read into some byte buffer.
    async fn read_bytes(&self, offset: Offset, buf: & mut [u8]) -> Result<usize, Error>;

}

/// Trait extension to simplify reading.
pub trait ReadExt: Read {
    /// Read a slice of sized objects.
    async fn read_slice<T: Sized>(&self, offset: Offset, buf: &mut [T]) -> Result<usize, Error> {
        let x = unsafe {
            core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, core::mem::size_of_val(buf))
        };
        let res = self.read_bytes(offset, x).await?;
        Ok(res / core::mem::size_of::<T>())
    }

    /// Read a single object.
    async fn read_object<T: Sized>(&self, offset: Offset) -> Result<T, Error> {
        let mut res = MaybeUninit::uninit();
        let x = unsafe {
            core::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, core::mem::size_of::<T>())
        };
        let n = self.read_bytes(offset, x).await?;
        if n != x.len() {
            return Err(anyhow::anyhow!("partial read"));
        }
        Ok(unsafe { res.assume_init() })
    }
}
