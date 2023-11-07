//! File in VFAT

use super::structs::DirEntry;

#[derive(Debug)]
pub struct File {
    inode: DirEntry,
}


impl File {
    pub (crate) fn new(inode: DirEntry) -> Self {
        Self { inode }
    }
}
