//! File support.

use super::{Dir, Error, Ext4Fs, FileType, Inode, Offset, Read, ReadExt};
use ap_storage::{file::File, meta::MetaData};
use ap_storage_ext4::dir::DirEntryHeader;
use core::cell::RefCell;

pub struct Ext4File<'a> {
    block_size: u64,
    fs: &'a Ext4Fs<'a>,
    inode: Inode,
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
    /// The nnumber of adjacent blocks merged.
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
    fn search_block(&self, mut blk: u64) -> Result<(u64, u64), Error> {
        if blk < 12 {
            let start = self.inode.blocks[blk as usize] as u64;
            let cnt = Self::count_contigous(&self.inode.blocks[blk as usize..12]);
            return Ok((start, cnt as u64));
        }
        blk -= 12;
        let log_numbers_per_block = self.block_size.ilog2() as usize - 2;
        let index_at_level = |blk: u64, level: usize| {
            (blk >> ((level - 1) * log_numbers_per_block)) & ((1 << log_numbers_per_block) - 1)
        };

        // find required level and adjust blk accordingly
        let mut level = 1usize;
        while level < 4 {
            if blk >> (level * log_numbers_per_block) == 0 {
                break;
            }
            blk -= 1 << (level * log_numbers_per_block);
            level += 1;
        }

        // follow level indirections
        let mut res = self.inode.blocks[11 + level];
        let mut cnt = 1;
        while level > 0 && res != 0 {
            let index = index_at_level(blk, level);
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
            cnt = blocks_at_level - blk % blocks_at_level;
        }
        Ok((res as u64, cnt))
    }

    /// Get an extent object at the certain disk offset.
    fn get_extent<X: Sized + Copy>(&self, ofs: u64) -> Result<X, Error> {
        match ofs {
            // The first extents are inline in the block.
            0..=48 => Ok(unsafe {
                *(self.inode.extent().unwrap().as_ptr().add(ofs as usize / 4) as *const X)
            }),
            // Could detect errors here
            _ => self.fs.disk.read_object(ofs),
        }
    }

    /// Do a binary search for a block in the extend tree.
    fn search_binary(&self, block: u64, ofs: u64, count: usize) -> Result<u64, Error> {
        let mut left = 0;
        let mut right = count - 1;

        while left < right {
            let middle = (left + right + 1) / 2;
            let start = self.get_extent::<u32>(ofs + middle as u64 * 12)? as u64;
            if start <= block {
                left = middle;
                if start == block {
                    break;
                }
            } else {
                right = middle - 1;
            }
        }
        Ok(ofs + left as u64 * 12)
    }

    /// Search in the extent tree for the right block.
    ///
    /// Returns the physical block number and the number of continous blocks.
    /// A zero block number means a hole in the file.
    fn search_extent(&self, block: u64) -> Result<(u64, u64), Error> {
        let mut ofs = 0;
        let mut depth = 0;

        loop {
            let header: Ext4ExtentHeader = self.get_extent(ofs)?;
            if header.magic != 0xf30a {
                return Err(anyhow::anyhow!("extent magic"));
            }
            if ofs != 0 && depth != header.depth + 1 {
                return Err(anyhow::anyhow!("extent depth"));
            }
            if header.depth == 0 {
                ofs = self.search_binary(block, ofs + 12, header.entries as usize)?;
                let entry: Ext4ExtentLeaf = self.get_extent(ofs)?;
                if entry.block as u64 > block {
                    // hole before
                    let n = entry.block as u64 - block;
                    return Ok((0, n));
                }
                let n = block - entry.block as u64;
                if n < entry.len as u64 {
                    return Ok((entry.dest() + n, entry.len as u64 - n));
                }
                // hole after
                return Ok((0, 1));
            }
            depth = header.depth;
            ofs = self.search_binary(block, ofs + 12, header.entries as usize)?;
            let entry: Ext4ExtentIndex = self.get_extent(ofs)?;
            if entry.block as u64 > block {
                let n = entry.block as u64 - block;
                return Ok((0, n));
            }
            ofs = entry.dest() * self.block_size;
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
        let res = if cfg!(feature="file_extents") && self.inode.extent().is_some() {
            self.search_extent(block_in_file)?
        } else if cfg!(feature="file_blocks") {
            self.search_block(block_in_file)?
        } else {
            return Err(anyhow::anyhow!("missing feature to lookup blocks"));
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

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Ext4ExtentHeader {
    magic: u16,
    entries: u16,
    _0: u16,
    depth: u16,
    _1: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Ext4ExtentLeaf {
    block: u32,
    len: u16,
    hi: u16,
    lo: u32,
}

impl Ext4ExtentLeaf {
    fn dest(&self) -> u64 {
        ((self.hi as u64) << 32) | self.lo as u64
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Ext4ExtentIndex {
    block: u32,
    lo: u32,
    hi: u16,
    _0: u16,
}

impl Ext4ExtentIndex {
    fn dest(&self) -> u64 {
        ((self.hi as u64) << 32) | self.lo as u64
    }
}
