//! Directory iterator.
use super::{Error, Read};

/// A directory iterator.
pub struct DirIterator<'a, T, const N: usize> {
    parent: &'a T,
    offset: u64,
    start: usize,
    end: usize,
    buf: [u8; N],
}

impl<'a, T: Read, const N: usize> DirIterator<'a, T, N> {
    pub fn new(parent: &'a T) -> Self {
        Self {
            parent,
            offset: 0,
            start: 0,
            end: 0,
            buf: [0; N],
        }
    }

    pub async fn next<'b>(
        &'b mut self,
        name: &mut [u8; 255],
    ) -> Result<Option<DirEntryHeader>, Error> {
        const O: usize = core::mem::size_of::<DirEntryHeader>();

        // read more data?
        let n = self.end - self.start;
        if n < O {
            self.buf[..n].copy_within(self.start..self.end, 0);
            self.start = 0;
            self.end = self.parent.read_bytes(self.offset, &mut self.buf).await?;
            self.offset += self.end as u64;
        }
        // still smaller as the header?
        if self.end - self.start < O {
            return Ok(None);
        }
        let header = unsafe { *(self.buf.as_ptr().add(self.start) as *const DirEntryHeader) };

        // not enough for the name?
        let name_len = header.name_len as usize;
        if self.end - self.start < O + name_len {
            self.end += self
                .parent
                .read_bytes(self.offset, &mut self.buf[self.end..])
                .await?;
            self.offset += self.end as u64;
        }

        // truncated entry
        if self.end - self.start < O + name_len {
            return Err(anyhow::anyhow!("truncated dir"));
        }

        // copy-out the name
        name[..name_len].copy_from_slice(&self.buf[self.start + O..self.start + O + name_len]);

        // drop the entry
        self.start += header.rec_len as usize;
        Ok(Some(header))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DirEntryHeader {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
}

impl DirEntryHeader {
    pub fn is_dir(&self) -> bool {
        self.file_type == 2
    }
    pub fn inode(&self) -> usize {
        self.inode as usize
    }
}
