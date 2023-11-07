//! File in VFAT

use super::{structs::DirEntry, FatFs};
use ap_storage::{Error, Offset, Read};
use core::cell::RefCell;

#[derive(Debug)]
pub struct File<'a> {
    fs: &'a FatFs<'a>,
    inode: DirEntry,
    cache: RefCell<FileCache>,
}

/// The in-file cache to speedup linear reads.
#[derive(Debug, Default)]
struct FileCache {
    block: u32,
    cluster: u32,
}


impl<'a> File<'a> {
    /// Creating a file from a directory entry.
    pub(crate) fn new(fs: &'a FatFs<'a>, inode: DirEntry) -> Self {
        Self {
            inode,
            fs,
            cache: Default::default(),
        }
    }
}

impl Read for File<'_> {
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.inode.size();
        if offset >= size.into() {
            return Ok(0);
        }

        let max_n = core::cmp::min(buf.len(), size as usize - offset as usize);
        let block = (offset / self.fs.block_size as Offset) as u32;
        let offset_in_block = offset % self.fs.block_size as Offset;

        let mut cache = self.cache.borrow_mut();

        // rewind?
        if cache.block > block || cache.cluster == 0 {
            cache.block = 0;
            cache.cluster = self.inode.cluster();
        }

        // follow the FAT for the right block
        while cache.block != block {
            cache.cluster = self.fs.follow_fat(cache.cluster)?;

            // EOF or bad clusters?
            if cache.cluster >= self.fs.fat_mask - 8 {
                return Ok(0);
            }
            cache.block += 1;
        }
        // limit the bytes to the current block
        let max_n = core::cmp::min(
            max_n,
            self.fs.block_size as usize - offset_in_block as usize,
        );
        let ofs = cache.cluster as u64 * self.fs.block_size as u64 + offset_in_block;
        self.fs.disk.read_bytes(ofs, &mut buf[..max_n])
    }
}
