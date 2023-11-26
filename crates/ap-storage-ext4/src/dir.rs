//! On-disk directory entry.

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DirEntryHeader {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
}

impl DirEntryHeader {
    pub fn inode(&self) -> u64 {
        self.inode as u64
    }
}
