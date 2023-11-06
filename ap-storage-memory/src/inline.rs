//! A cache with inline backing-store.

use ap_storage::{Error, Offset, Read};

pub struct InlineCacheImpl<'a, const N: usize> {
    parent: &'a dyn Read,
    /// A buffer holding upto N bytes.
    buffer: [u8; N],
    /// The offset.
    offset: Offset,
    /// Valid bytes in the cache.
    valid: usize,
}

impl<'a, const N: usize> InlineCacheImpl<'a, N> {
    /// Create a new instance.
    pub fn new(parent: &'a dyn Read) -> Self {
        Self {
            parent,
            buffer: [0; N],
            offset: 0,
            valid: 0,
        }
    }
    /// Read from the internal buffer. Returns the bytes read.
    fn read_from_buffer(&self, ofs: Offset, buf: &mut [u8]) -> usize {
        if ofs < self.offset || ofs >= self.offset + self.valid as u64 {
            return 0;
        }
        let skip = (ofs - self.offset) as usize;
        let res = core::cmp::min(self.valid - skip, buf.len());
        buf[..res].copy_from_slice(&self.buffer[skip..skip + res]);
        res
    }

    /// Read bytes from a mutable self.
    pub fn read_mut(&mut self, ofs: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let res = self.read_from_buffer(ofs, buf);
        if res > 0 {
            return Ok(res);
        }

        let ofs_in_block = ofs % N as u64;

        // crossing a block boundary - it does not make sense to cache the previous block
        if ofs_in_block as usize + buf.len() > N {
            return self.parent.read_bytes(ofs, buf);
        }

        // read a whole block
        self.offset = ofs - ofs_in_block;
        self.valid = self
            .parent
            .read_bytes(ofs - ofs_in_block, &mut self.buffer)?;
        Ok(self.read_from_buffer(ofs, buf))
    }
}
