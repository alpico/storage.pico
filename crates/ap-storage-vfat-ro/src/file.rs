//! File in VFAT

use super::{attr::Attr, dir::Dir, DirectoryEntry, VFatFS};
use ap_storage::{
    meta::{FileType, MetaData},
    Error, Offset, Read, ReadExt,
};
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct File<'a> {
    pub(crate) fs: &'a VFatFS<'a>,
    pub(crate) inode: DirectoryEntry,
    id: Offset,
    cache: RefCell<FileCache>,
}

/// The in-file cache to speedup linear reads.
#[derive(Debug, Default, Clone)]
struct FileCache {
    block: u32,
    cluster: u32,
    last_offset: u64,
}

impl<'a> File<'a> {
    /// Creating a file from a directory entry.
    pub(crate) fn new(fs: &'a VFatFS<'a>, inode: DirectoryEntry, id: Offset) -> Self {
        Self {
            inode,
            fs,
            id,
            cache: Default::default(),
        }
    }
    /// Is the root directory.
    pub fn is_root(&self) -> bool {
        self.inode.cluster() == 0
    }

    fn size(&self) -> Offset {
        let res = self.inode.size();
        if !self.inode.is_dir() || res < 2 << 20 {
            return res;
        }
        // directories do not have a valid size.  Follow the FAT to calculate the value.
        let mut cluster = self.inode.cluster();
        let mut res = 0;
        while cluster < self.fs.fat_mask - 8 && res < 2 << 20 {
            res += self.fs.cluster_size;
            cluster = self.fs.follow_fat(cluster).unwrap_or(!0u32);
        }
        res as Offset
    }
}

impl<'a> ap_storage::file::File for File<'a> {
    type DirType<'c> = Dir<'c> where Self: 'c;
    type AttrType<'c> = Attr<'c> where Self: 'c;

    fn dir(&self) -> Option<Self::DirType<'_>> {
        if self.inode.is_dir() {
            return Some(Dir::new(self));
        }
        None
    }

    fn attr(&self) -> Self::AttrType<'_> {
        Attr::new(self)
    }

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
        let id = if entry.is_dir() {
            // for directories it is the start of the contents - this ensures hard-links have the same id
            entry.cluster() as u64 * self.fs.cluster_size as u64
        } else {
            // For disks it is the offset of the directory entry - this ensures empty files still get a unique id
            //
            // The `| 1` is not strictly necessary, as all directories should start with a self-pointer so there should never be overlapping of values
            self.cache.borrow().last_offset | 1
        };

        Ok(Self::new(self.fs, entry, id))
    }

    fn meta(&self) -> MetaData {
        let filetype = if self.inode.attr & 0x8 != 0 || self.inode.name[0] == 0xe5 {
            FileType::Unknown
        } else if self.inode.is_dir() {
            FileType::Directory
        } else {
            FileType::File
        };
        MetaData {
            size: self.size(),
            filetype,
            id: self.id,
            mtime: self.inode.mtime(),
        }
    }
}

impl Read for File<'_> {
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.inode.size();
        if offset >= size {
            return Ok(0);
        }

        let max_n = core::cmp::min(buf.len(), size as usize - offset as usize);
        let block = (offset / self.fs.cluster_size as Offset) as u32;
        let offset_in_block = offset % self.fs.cluster_size as Offset;

        let mut cache = self.cache.borrow_mut();

        // rewind?
        if cache.block > block || cache.cluster == 0 {
            cache.block = 0;
            cache.cluster = self.inode.cluster();
        }

        let ofs = {
            // root-directory on fat12+16 is in its own region
            if cache.cluster == 0 && self.fs.root_size != 0 {
                self.fs.root_start
            } else {
                if cache.cluster == 0 {
                    cache.cluster = self.fs.root_cluster;
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
                (cache.cluster as u64 - 2) * self.fs.cluster_size as u64 + self.fs.data_start
            }
        };

        // keep the last offset read in the cache to get the file-id
        cache.last_offset = ofs + offset_in_block;

        // limit the bytes to the current block
        let max_n = core::cmp::min(max_n, self.fs.cluster_size as usize - offset_in_block as usize);
        self.fs.disk.read_bytes(ofs + offset_in_block, &mut buf[..max_n])
    }
}
