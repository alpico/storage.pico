//! Traits for reading.
use crate::{Error, Offset, msg2err};
use core::mem::MaybeUninit;

/// Read from a certain offset into a buffer.
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

    /// Get the size.
    fn detect_size(&self) -> Offset;
}

impl ReadExt for &dyn Read {
    fn read_exact(&self, offset: Offset, buf: &mut [u8]) -> Result<(), Error> {
        let mut n = 0;
        while n != buf.len() {
            match self.read_bytes(offset + n as Offset, &mut buf[n..])? {
                0 => return Err(msg2err!(PartialReadError)),
                c => n += c,
            }
        }
        Ok(())
    }
    fn read_object<T: Sized>(&self, offset: Offset) -> Result<T, Error> {
        let mut res = MaybeUninit::uninit();
        let buf = unsafe { core::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, core::mem::size_of::<T>()) };

        self.read_exact(offset, buf)?;
        Ok(unsafe { res.assume_init() })
    }

    /// Detect the size of a disk by doing binary search.
    fn detect_size(&self) -> Offset {
        let mut start = 0;
        let mut end = !0;
        while start != end {
            let middle = (start + end) / 2;
            let mut buf = [0u8];
            match self.read_bytes(middle, &mut buf) {
                Ok(1) => {
                    start = middle;
                }
                _ => {
                    end = middle;
                }
            }
            if middle == (start + end) / 2 {
                break;
            }
        }
        end
    }
}

/// An exact read could only be partially done.
#[derive(Debug)]
pub struct PartialReadError;

impl core::fmt::Display for PartialReadError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}
