//! Traits for writing.
use crate::{Error, Offset};

/// Write to a file or disk at a certain offset.
pub trait Write {
    /// Write some byte buffer.
    fn write_bytes(&self, offset: Offset, buf: &[u8]) -> Result<usize, Error>;
    /// Discard a region and return the bytes successfully descarded.
    fn discard(&self, offset: Offset, len: Offset) -> Result<Offset, Error>;
}

/// Trait extension to simplify writing.
pub trait WriteExt {
    /// Write the whole buffer.
    fn write_exact(&self, offset: Offset, buf: &[u8]) -> Result<(), Error>;
    /// Write the whole object.
    fn write_object<T: Sized>(&self, offset: Offset, obj: T) -> Result<(), Error>;
    /// Discard a whole area.
    fn discard_all(&self, offset: Offset, len: Offset) -> Result<(), Error>;
}

impl WriteExt for &dyn Write {
    /// Write the whole buffer.
    fn write_exact(&self, offset: Offset, buf: &[u8]) -> Result<(), Error> {
        let mut done = 0;
        while done != buf.len() {
            match self.write_bytes(offset + done as Offset, &buf[done..])? {
                0 => return Err(Error::msg(PartialWriteError)),
                n => done += n,
            }
        }
        Ok(())
    }

    /// Write the whole object
    fn write_object<T: Sized>(&self, offset: Offset, obj: T) -> Result<(), Error> {
        let buf = unsafe { core::slice::from_raw_parts(&obj as *const T as *const u8, core::mem::size_of::<T>()) };
        self.write_exact(offset, buf)
    }

    /// Discard all bytes in the range.
    fn discard_all(&self, mut offset: Offset, len: Offset) -> Result<(), Error> {
        let end = offset + len;
        while offset < len {
            match self.discard(offset, end - offset)? {
                0 => return Err(Error::msg(PartialWriteError)),
                n => offset += n,
            }
        }
        Ok(())
    }
}

/// An exact write could only be partially done.
#[derive(thiserror::Error, Debug)]
pub struct PartialWriteError;

impl core::fmt::Display for PartialWriteError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}
