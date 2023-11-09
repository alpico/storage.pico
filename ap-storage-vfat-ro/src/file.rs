//! File in VFAT

use super::{dir::Dir, DirectoryEntry, VFatFS};
use ap_storage::{directory::Iterator, Error, Offset, Read, ReadExt};
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct File<'a> {
    pub(crate) fs: &'a VFatFS<'a>,
    pub(crate) inode: DirectoryEntry,
    cache: RefCell<FileCache>,
}

/// The in-file cache to speedup linear reads.
#[derive(Debug, Default, Clone)]
struct FileCache {
    block: u32,
    cluster: u32,
}

impl<'a> File<'a> {
    /// Creating a file from a directory entry.
    pub(crate) fn new(fs: &'a VFatFS<'a>, inode: DirectoryEntry) -> Self {
        Self {
            inode,
            fs,
            cache: Default::default(),
        }
    }
    /// Is the root directory.
    pub fn is_root(&self) -> bool {
        self.inode.cluster() == 0
    }
}

impl<'a> ap_storage::file::File for File<'a> {
    /// Open a file relative to the given directory.
    fn open(&self, mut offset: Offset) -> Result<Self, Error> {
        if !self.inode.is_dir() {
            return Err(anyhow::anyhow!("not a directory"));
        }
        if self.is_root() {
            if offset < 2 {
                return Ok(self.clone());
            }
            offset -= 2;
        }

        let entry: DirectoryEntry = (self as &dyn Read).read_object(32 * offset)?;
        Ok(Self::new(self.fs, entry))
    }

    /// Return a directory iterator.
    fn dir(&self) -> Option<impl Iterator> {
        if self.inode.is_dir() {
            return Some(Dir::new(self));
        }
        None
    }
    fn size(&self) -> Offset {
        // XXX measure on directories
        self.inode.size().into()
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

        // root-directory on
        let ofs = {
            if cache.cluster == 0 {
                self.fs.root_start
            } else {
                // follow the FAT for the right block

                while cache.block != block {
                    cache.cluster = self.fs.follow_fat(cache.cluster)?;

                    // EOF or bad clusters?
                    if cache.cluster >= self.fs.fat_mask - 8 {
                        return Ok(0);
                    }
                    cache.block += 1;
                }
                (cache.cluster as u64 - 2) * self.fs.block_size as u64 + self.fs.data_start
            }
        };
        // limit the bytes to the current block
        let max_n = core::cmp::min(
            max_n,
            self.fs.block_size as usize - offset_in_block as usize,
        );
        self.fs
            .disk
            .read_bytes(ofs + offset_in_block, &mut buf[..max_n])
    }
}
