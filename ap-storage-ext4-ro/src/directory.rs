//! Directory iterator.
use super::{Error, Read, read_object};

/// A directory iterator.
pub struct DirIterator<'a> {
    parent: &'a dyn Read,
    offset: u64,
}

impl<'a> DirIterator<'a> {
    pub fn new(parent: &'a dyn Read) -> Self {
        Self { parent, offset: 0 }
    }

    pub fn next(&mut self, name: &mut [u8; 255]) -> Result<DirEntryHeader, Error> {
        const O: usize = core::mem::size_of::<DirEntryHeader>();

        let header: DirEntryHeader = read_object(self.parent, self.offset)?;
        let name_len = header.name_len as usize;

        let n = self
            .parent
            .read_bytes(self.offset + O as u64, &mut name[..name_len])?;
        if n < name_len {
            return Err(anyhow::anyhow!("truncated dir"));
        }
        self.offset += header.rec_len as u64;
        Ok(header)
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
    pub fn inode(&self) -> u64 {
        self.inode as u64
    }

    pub fn name_len(&self) -> usize {
        self.name_len as usize
    }
}
