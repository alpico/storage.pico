//! Traits for reading.
use crate::{Offset, Error};
use core::mem::MaybeUninit;

/// Trait to read from a file or disk from a certain offset.
pub trait Read {
    /// Read into some byte buffer. Returning zero means EOF.
    fn read_bytes(& self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error>;

}

/// Trait extension to simplify reading.
pub trait ReadExt: Read {
    /// Read a single object.  Repeat partial reads until EOF or error.
    fn read_object<T: Sized>(&self, offset: Offset) -> Result<T, Error> {
        let mut res = MaybeUninit::uninit();
        let x = unsafe {
            core::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, core::mem::size_of::<T>())
        };

        let mut n = 0;
        while n != x.len() {
            match self.read_bytes(offset + n as u64, &mut x[n..])? {
                0 => return Err(anyhow::anyhow!("partial read")),
                c =>  { n +=c },
            }
        }
        Ok(unsafe { res.assume_init() })
    }
}
