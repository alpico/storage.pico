//! File support.

use super::{Dir, Error, Ext4Fs, FileType, Inode, Offset, Read, ReadExt};
use ap_storage::{file::File, meta::MetaData};
use ap_storage_ext4::dir::DirEntryHeader;
use core::cell::RefCell;

pub struct Ext4File<'a> {
    pub (crate) block_size: u64,
    pub (crate) fs: &'a Ext4Fs<'a>,
    pub (crate) inode: Inode,
    leaf_optimization: bool,
    nr: u64,
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
    /// The number of adjacent blocks merged.
    #[cfg(feature = "file_blocks")]
    const MAX_MERGED: usize = 16;

    /// Open the given file by inode number.
    pub fn new(fs: &'a Ext4Fs, nr: u64) -> Result<Self, Error> {
        let inode = fs.inode(nr)?;
        Ok(Self {
            block_size: fs.sb.block_size(),
            fs,
            inode,
            leaf_optimization: fs.leaf_optimization,
            nr,
            cache: Default::default(),
        })
    }

    /// Count the contigious block numbers in the slice.
    #[cfg(feature = "file_blocks")]
    fn count_contigous(v: &[u32]) -> usize {
        let mut cnt = 1;
        let start = v[0] as u64;
        while cnt < v.len() {
            if start != 0 && (v[cnt] as u64) != start + cnt as u64 || start == 0 && v[cnt] != 0 {
                break;
            }
            cnt += 1;
        }
        cnt
    }

    /// Search in the (indirect) blocks for the right block.
    ///
    /// Returns the physical block number and the number of continous blocks.
    /// A zero block number means a hole in the file.
    fn search_block(&self, mut _block: u64) -> Result<(u64, u64), Error> {
        #[cfg(not(feature = "file_blocks"))]
        return Err(anyhow::anyhow!("blocks not supported"));

        #[cfg(feature = "file_blocks")]
        {
            if _block < 12 {
                let start = self.inode.blocks[_block as usize] as u64;
                let cnt = Self::count_contigous(&self.inode.blocks[_block as usize..12]);
                return Ok((start, cnt as u64));
            }
            _block -= 12;
            let log_numbers_per_block = self.block_size.ilog2() as usize - 2;
            let index_at_level = |blk: u64, level: usize| {
                (blk >> ((level - 1) * log_numbers_per_block)) & ((1 << log_numbers_per_block) - 1)
            };

            // find required level and adjust _block accordingly
            let mut level = 1usize;
            while level < 4 {
                if _block >> (level * log_numbers_per_block) == 0 {
                    break;
                }
                _block -= 1 << (level * log_numbers_per_block);
                level += 1;
            }

            // follow level indirections
            let mut res = self.inode.blocks[11 + level];
            let mut cnt = 1;
            while level > 0 && res != 0 {
                let index = index_at_level(_block, level);
                if level != 1 || (index + Self::MAX_MERGED as u64) >> log_numbers_per_block > 0 {
                    res = self
                        .fs
                        .disk
                        .read_object(res as u64 * self.block_size + index * 4)?;
                } else {
                    let blocks: [u32; Ext4File::MAX_MERGED] = self
                        .fs
                        .disk
                        .read_object(res as u64 * self.block_size + index * 4)?;
                    res = blocks[0];
                    cnt = Self::count_contigous(&blocks);
                }
                level -= 1
            }

            // huge gap?
            let mut cnt = cnt as u64;
            if level > 0 {
                let blocks_at_level = 1 << (level * log_numbers_per_block);
                cnt = blocks_at_level - _block % blocks_at_level;
            }
            Ok((res as u64, cnt))
        }
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
            self.search_block(block_in_file)?
        };
        cache.block = block_in_file;
        cache.phys = res.0;
        cache.cnt = res.1;
        Ok(res)
    }
}

impl<'a> File for Ext4File<'a> {
    /// Return a iterator if this is a directory.
    type DirType<'c> = Dir<'c> where Self: 'c;
    fn dir(&self) -> Option<Self::DirType<'_>> {
        if self.inode.ftype() == FileType::Directory
            && (self.inode.version != 1 || !self.leaf_optimization)
        {
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
            size: self.inode.size(),
            filetype: self.inode.ftype(),
            id: self.nr,
            mtime: self.inode.mtime(),
        }
    }
}

impl<'a> Read for Ext4File<'a> {
    /// Read in the given inode.
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.inode.size();

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

        let block_in_file = offset / self.block_size;
        let offset_in_block = offset % self.block_size;

        let (phys, max_blocks) = self.lookup_block(block_in_file)?;

        let valid_size = core::cmp::min(
            valid_size as Offset,
            max_blocks * self.block_size - offset_in_block,
        ) as usize;
        let buf = &mut buf[..valid_size];
        if phys == 0 {
            buf.fill(0);
            return Ok(valid_size);
        }
        let ofs = phys * self.block_size + offset_in_block;
        self.fs.disk.read_bytes(ofs, buf)
    }
}
