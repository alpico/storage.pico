use crate::file::Ext4File;
use ap_storage::Error;
pub struct Ext4Extents<'a>(pub &'a Ext4File<'a>);

#[cfg(not(feature = "file_extents"))]
impl<'a> Ext4Extents<'a> {
    pub fn search(&self, _block: u64) -> Result<(u64, u64), Error> {
        Err(anyhow::anyhow!("extents not supported"))
    }
}
#[cfg(feature = "file_extents")]
use ap_storage_ext4::extent::*;

#[cfg(feature = "file_extents")]
impl<'a> Ext4Extents<'a> {
    /// Get an extent object at the certain disk offset.
    fn get<X: Sized + Copy>(&self, ofs: u64) -> Result<X, Error> {
        use ap_storage::ReadExt;
        match ofs {
            // The first extents are inline in the block.
            0..=48 => Ok(unsafe {
                *(self
                    .0
                    .inode
                    .extent()
                    .unwrap()
                    .as_ptr()
                    .add(ofs as usize / 4) as *const X)
            }),
            // Could detect errors here
            _ => self.0.fs.disk.read_object(ofs),
        }
    }

    /// Do a binary search for a block in the extend tree.
    fn search_binary(&self, block: u64, ofs: u64, count: usize) -> Result<u64, Error> {
        let mut left = 0;
        let mut right = count - 1;

        while left < right {
            let middle = (left + right + 1) / 2;
            let start = self.get::<u32>(ofs + middle as u64 * 12)? as u64;
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
    pub fn search(&self, block: u64) -> Result<(u64, u64), Error> {
        let mut ofs = 0;
        let mut depth = 0;
        loop {
            let header: Ext4ExtentHeader = self.get(ofs)?;
            if header.magic != 0xf30a {
                return Err(anyhow::anyhow!("extent magic"));
            }
            if ofs != 0 && depth != header.depth + 1 {
                return Err(anyhow::anyhow!("extent depth"));
            }
            if header.depth == 0 {
                ofs = self.search_binary(block, ofs + 12, header.entries as usize)?;
                let entry: Ext4ExtentLeaf = self.get(ofs)?;
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
            let entry: Ext4ExtentIndex = self.get(ofs)?;
            if entry.block as u64 > block {
                let n = entry.block as u64 - block;
                return Ok((0, n));
            }
            ofs = entry.dest() * self.0.block_size;
        }
    }
}
