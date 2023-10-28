//! File support.

use super::{DirIterator, Error, Ext4Fs, FileType, Inode, Offset, Read, ReadExt};

pub struct File<'a, T> {
    inode: Inode,
    fs: &'a Ext4Fs<'a, T>,
}

impl<'a, T: ReadExt> File<'a, T> {
    /// Open the given file by inode number.
    pub fn new(fs: &'a Ext4Fs<T>, nr: u64) -> Result<Self, Error> {
        Ok(Self {
            inode: fs.inode(nr)?,
            fs,
        })
    }

    /// Open from this file.
    pub fn open(&self, nr: u64) -> Result<Self, Error> {
        Ok(Self {
            inode: self.fs.inode(nr)?,
            fs: self.fs,
        })
    }

    pub fn size(&self) -> u64 {
        self.inode.size()
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
    fn search_extent(&self, block: u64, block_size: u64) -> Result<(u64, u64), Error> {
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
                    return Ok((entry.dest(), entry.len as u64 - n));
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
            ofs = entry.dest() * block_size;
        }
    }

    /// Return a iterator if this is a directory.
    ///
    /// The optimization drops empty directories that have not been updated yet..
    /// XXX this field should be moved into the filesystem object
    pub fn dir(&self, optimization: bool) -> Option<DirIterator<Self>> {
        if self.inode.ftype() == FileType::Directory && (self.inode.version != 1 || !optimization) {
            return Some(DirIterator::new(self));
        }
        None
    }
}

impl<'a, T: ReadExt> Read for File<'a, T> {
    /// Read in the given inode.
    fn read_bytes(&self, offset: Offset, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.inode.size();

        // check for eof
        if offset >= size {
            return Ok(0);
        }
        let valid_size = core::cmp::min(size - offset, buf.len() as u64) as usize;

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

        let block_in_file = offset / self.fs.sb.block_size();
        let offset_in_block = offset % self.fs.sb.block_size();

        let (phys, max_blocks) = {
            if self.inode.extent().is_some() {
                self.search_extent(block_in_file, self.fs.sb.block_size())?
            } else {
                todo!("original blocks");
            }
        };

        let valid_size = core::cmp::min(
            valid_size as u64,
            max_blocks * self.fs.sb.block_size() - offset_in_block,
        ) as usize;
        let buf = &mut buf[..valid_size];
        if phys == 0 {
            buf.fill(0);
            return Ok(valid_size);
        }
        let ofs = phys * self.fs.sb.block_size() + offset_in_block;
        self.fs.disk.read_bytes(ofs, buf)
    }
}

/// Also provide ReadExt functionality.
impl<'a, T: ReadExt> ReadExt for File<'a, T> {}

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
