use super::{Error, Offset, Read};

/// Read from a slice of memory.
#[derive(Clone, Copy)]
pub struct ReadSlice<'a>(pub &'a [u8]);

impl Read for ReadSlice<'_> {
    fn read_bytes(&self, ofs: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        if ofs >= self.0.len() as Offset {
            return Ok(0);
        }
        let ofs = ofs as usize;
        let n = core::cmp::min(self.0.len() - ofs, buf.len());
        buf[..n].copy_from_slice(&self.0[ofs..ofs + n]);
        Ok(n)
    }
}
