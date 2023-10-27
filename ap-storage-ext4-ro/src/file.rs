//! File support.

use super::{DirIterator, Error, Ext4Fs, FileType, Inode, Read, ReadExt};

pub struct File<'a, T> {
    inode: Inode,
    fs: &'a Ext4Fs<T>,
}

impl<'a, T: ReadExt> File<'a, T> {
    /// Open the given file by inode number.
    pub async fn new(fs: &'a Ext4Fs<T>, nr: u64) -> Result<Self, Error> {
        let inode = fs.inode(nr).await?;
        Ok(Self { inode, fs })
    }
    /// Search in the extent tree for the right block.
    ///
    /// Returns the physical block number and the number of continous blocks.
    async fn search_extent(
        &self,
        mut data: &[u32],
        block: u64,
        block_size: u64,
    ) -> Result<(u64, u64), Error> {
        let mut buf = Vec::with_capacity(block_size as usize / 4);
        loop {
            assert!(data.len() >= 3);
            let header = unsafe { &*(data.as_ptr() as *const Ext4ExtentHeader) };
            if header.magic != 0xf30a {
                return Err(anyhow::anyhow!("not an extend header"));
            }

            // do a linear search for now - a binary search would be a bit faster on larger trees
            for i in 1..(header.entries + 2) {
                if i > header.entries {
                    return Err(anyhow::anyhow!("end of extend"));
                }
                let entry =
                    unsafe { &*(data.as_ptr().offset(i as isize * 3) as *const Ext4ExtentLeaf) };
                if block < entry.block as u64 {
                    return Err(anyhow::anyhow!("eof"));
                }
                if header.depth == 0 {
                    let n = block - entry.block as u64;
                    if n < entry.len as u64 {
                        return Ok((entry.dest(), entry.len as u64 - n));
                    }
                } else {
                    let entry = unsafe {
                        &*(data.as_ptr().offset(i as isize * 3) as *const Ext4ExtentIndex)
                    };
                    let ofs = entry.dest() * block_size;
                    self.fs.disk.read_slice(ofs, &mut buf).await?;
                    data = &buf;
                    break;
                }
            }
        }
    }

    pub fn dir(&self, optimization: bool) -> Option<DirIterator<Self, 4096>> {
        if self.inode.ftype() == FileType::Directory && (self.inode.version != 1 || !optimization) {
            return Some(DirIterator::new(self));
        }
        None
    }
}

impl<'a, T: ReadExt> Read for File<'a, T> {
    /// Read in the given inode.
    async fn read_bytes(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.inode.size();

        // check for eof
        if offset >= size {
            return Ok(0);
        }

        // small symlinks are stored inline
        if self.inode.ftype() == FileType::SymLink && size < 60 {
            let n = core::cmp::min(size as usize, buf.len());
            buf[..n].copy_from_slice(unsafe {
                core::slice::from_raw_parts(&self.inode.blocks as *const u32 as *const u8, n)
            });
            return Ok(n);
        }

        let block = offset / self.fs.sb.block_size();
        let offset_in_block = offset % self.fs.sb.block_size();
        let valid_size = size - offset;

        let (phys, max_blocks) = {
            if let Some(data) = self.inode.extent() {
                self.search_extent(data, block, self.fs.sb.block_size())
                    .await?
            } else {
                todo!("original blocks");
            }
        };

        let ofs = phys * self.fs.sb.block_size() + offset_in_block;
        let valid_size = core::cmp::min(
            valid_size,
            max_blocks * self.fs.sb.block_size() - offset_in_block,
        );
        if valid_size < buf.len() as u64 {
            return Err(anyhow::anyhow!("access out of range"));
        }
        self.fs.disk.read_bytes(ofs, buf).await
    }
}

#[derive(Debug)]
#[repr(C)]
struct Ext4ExtentHeader {
    magic: u16,
    entries: u16,
    _0: u16,
    depth: u16,
    _1: u32,
}

#[derive(Debug)]
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

#[derive(Debug)]
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
