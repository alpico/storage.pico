//! Traits for reading.
use crate::{Error, Offset};
use core::mem::MaybeUninit;

/// Trait to read from a file or disk from a certain offset.
pub trait Read {
    /// Read into some byte buffer. Returning zero means EOF.
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error>;
}

/// Extension methods to make implementations easier.
pub trait ReadExt {
    /// Fill the buffer.
    fn read_exact(&self, offset: Offset, buf: &mut [u8]) -> Result<(), Error>;

    /// Read a whole object.
    fn read_object<T: Sized>(&self, offset: Offset) -> Result<T, Error>;
}

impl ReadExt for &dyn Read {
    fn read_exact(&self, offset: Offset, buf: &mut [u8]) -> Result<(), Error> {
        let mut n = 0;
        while n != buf.len() {
            match self.read_bytes(offset + n as Offset, &mut buf[n..])? {
                0 => return Err(PartialReadError.into()),
                c => n += c,
            }
        }
        Ok(())
    }
    fn read_object<T: Sized>(&self, offset: Offset) -> Result<T, Error> {
        let mut res = MaybeUninit::uninit();
        let buf = unsafe {
            core::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, core::mem::size_of::<T>())
        };

        self.read_exact(offset, buf)?;
        Ok(unsafe { res.assume_init() })
    }
}

#[derive(thiserror::Error, Debug)]
pub struct PartialReadError;

impl core::fmt::Display for PartialReadError {
    fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { todo!() }
}
