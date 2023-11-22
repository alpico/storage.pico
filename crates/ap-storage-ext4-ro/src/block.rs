use crate::file::Ext4File;
use ap_storage::Error;

pub struct Ext4Blocks<'a>(pub &'a Ext4File<'a>);

#[cfg(not(feature = "file_blocks"))]
impl<'a> Ext4Blocks<'a> {
    pub fn search(&self, mut _block: u64) -> Result<(u64, u64), Error> {
        Err(anyhow::anyhow!("blocks not supported"))
    }
}

/// Count the contigious block numbers in the slice.
#[cfg(feature = "file_blocks")]
impl<'a> Ext4Blocks<'a> {
    /// The number of adjacent blocks merged.
    const MAX_MERGED: usize = 16;

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
    pub fn search(&self, mut _block: u64) -> Result<(u64, u64), Error> {
        use ap_storage::ReadExt;
        if _block < 12 {
            let start = self.0.inode.blocks[_block as usize] as u64;
            let cnt = Self::count_contigous(&self.0.inode.blocks[_block as usize..12]);
            return Ok((start, cnt as u64));
        }
        _block -= 12;
        let block_size = self.0.fs.sb.block_size();
        let log_numbers_per_block = block_size.ilog2() as usize - 2;
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
        let mut res = self.0.inode.blocks[11 + level];
        let mut cnt = 1;
        while level > 0 && res != 0 {
            let index = index_at_level(_block, level);
            if level != 1 || (index + Self::MAX_MERGED as u64) >> log_numbers_per_block > 0 {
                res = self
                    .0
                    .fs
                    .disk
                    .read_object(res as u64 * block_size + index * 4)?;
            } else {
                let blocks: [u32; Ext4Blocks::MAX_MERGED] = self
                    .0
                    .fs
                    .disk
                    .read_object(res as u64 * block_size + index * 4)?;
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
