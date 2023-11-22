//! On-disk directory entry.

use ap_storage::meta::FileType;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DirEntryHeader {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
}

impl DirEntryHeader {
    pub fn typ(&self) -> FileType {
        match self.file_type {
            1 => FileType::File,
            2 => FileType::Directory,
            7 => FileType::SymLink,
            _ => FileType::Unknown,
        }
    }
    pub fn inode(&self) -> u64 {
        self.inode as u64
    }
}
