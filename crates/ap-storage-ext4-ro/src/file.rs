//! File support.

use super::{attr, Dir, Error, Ext4Fs, FileType, Inode, Offset, Read, ReadExt};
use ap_storage::{file::File, meta::MetaData};
use ap_storage_ext4::dir::DirEntryHeader;
use core::cell::RefCell;

pub struct Ext4File<'a> {
    pub(crate) fs: &'a Ext4Fs<'a>,
    pub(crate) inode: Inode,
    leaf_optimization: bool,
    pub(crate) nr: u64,
    cache: RefCell<FileCache>,
}

/// The in-file cache to speedup linear reads.
#[derive(Debug, Default, Clone)]
struct FileCache {
    block: u64,
    phys: u64,
    cnt: u64,
}

impl<'a> Ext4File<'a> {
    /// Open the given file by inode number.
    pub fn new(fs: &'a Ext4Fs, nr: u64) -> Result<Self, Error> {
        let inode = fs.inode(nr)?;
        Ok(Self {
            fs,
            inode,
            leaf_optimization: fs.leaf_optimization,
            nr,
            cache: Default::default(),
        })
    }

    fn lookup_block(&self, block_in_file: u64) -> Result<(u64, u64), Error> {
        let mut cache = self.cache.borrow_mut();
        if cache.block <= block_in_file && cache.block + cache.cnt > block_in_file {
            let ofs = block_in_file - cache.block;
            if cache.phys == 0 {
                return Ok((cache.phys, cache.cnt - ofs));
            }
            return Ok((cache.phys + ofs, cache.cnt - ofs));
        }
        let res = if self.inode.extent().is_some() {
            crate::extent::Ext4Extents(self).search(block_in_file)?
        } else {
            crate::block::Ext4Blocks(self).search(block_in_file)?
        };
        cache.block = block_in_file;
        cache.phys = res.0;
        cache.cnt = res.1;
        Ok(res)
    }
}

impl<'a> File for Ext4File<'a> {
    type AttrType<'c> = attr::Attr<'c> where Self: 'c;
    fn attr(&self) -> Self::AttrType<'_> {
        attr::Attr::new(self)
    }

    type DirType<'c> = Dir<'c> where Self: 'c;
    fn dir(&self) -> Option<Self::DirType<'_>> {
        if self.inode.ftype() == FileType::Directory && (self.inode.version() != 1 || !self.leaf_optimization) {
            return Some(Dir::new(self));
        }
        None
    }

    fn open(&self, offset: Offset) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if self.inode.ftype() != FileType::Directory {
            return Err(anyhow::anyhow!("not a directory"));
        }
        let header: DirEntryHeader = (self as &dyn Read).read_object(offset)?;
        Self::new(self.fs, header.inode())
    }

    fn meta(&self) -> MetaData {
        MetaData {
            size: self.inode.size(self.fs.sb.feature_incompat),
            filetype: self.inode.ftype(),
            id: self.nr,
        }
    }
}

impl<'a> Read for Ext4File<'a> {
    /// Read in the given inode.
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.inode.size(self.fs.sb.feature_incompat);

        // check for eof
        if offset >= size {
            return Ok(0);
        }
        let valid_size = core::cmp::min(size - offset, buf.len() as Offset) as usize;

        // small symlinks are stored inline
        if self.inode.ftype() == FileType::SymLink && size <= 60 {
            buf[..valid_size].copy_from_slice(unsafe {
                core::slice::from_raw_parts(
                    (self.inode.blocks.as_ptr() as *const u8).add(offset as usize),
                    valid_size,
                )
            });
            return Ok(valid_size);
        }

        let block_size = self.fs.sb.block_size();
        let block_in_file = offset / block_size;
        let offset_in_block = offset % block_size;

        let (phys, max_blocks) = self.lookup_block(block_in_file)?;

        let valid_size = core::cmp::min(valid_size as Offset, max_blocks * block_size - offset_in_block) as usize;
        let buf = &mut buf[..valid_size];
        if phys == 0 {
            buf.fill(0);
            return Ok(valid_size);
        }
        let ofs = phys * block_size + offset_in_block;
        self.fs.disk.read_bytes(ofs, buf)
    }
}
