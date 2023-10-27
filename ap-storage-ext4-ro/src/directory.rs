//! Directory iterator.
use super::{Error, ReadExt};

/// A directory iterator.
pub struct DirIterator<'a, T> {
    parent: &'a T,
    offset: u64,
}

impl<'a, T: ReadExt> DirIterator<'a, T> {
    pub fn new(parent: &'a T) -> Self {
        Self { parent, offset: 0 }
    }

    pub async fn next<'b>(&'b mut self, name: &mut [u8; 255]) -> Result<DirEntryHeader, Error> {
        const O: usize = core::mem::size_of::<DirEntryHeader>();

        let header: DirEntryHeader = self.parent.read_object(self.offset).await?;
        let name_len = header.name_len as usize;

        let n = self
            .parent
            .read_bytes(self.offset + O as u64, &mut name[..name_len])
            .await?;
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
    pub fn inode(&self) -> usize {
        self.inode as usize
    }
}
