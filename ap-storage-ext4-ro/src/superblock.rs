//! Superblock definition.

/// Sparse superblock definition.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SuperBlock {
    pub inode_count: u32,
    _1: [u32; 4],
    first_block: u32,
    log_block_size: u32,
    _2: u32,
    pub blocks_per_group: u32,
    _3: u32,
    pub inodes_per_group: u32,
    _4: [u32; 3],
    pub magic: u16,
    _5: [u16; 15],
    _inode_size: u16,
    _6: [u16; 3],
    pub feature_incompat: u32,
    pub feature_ro_compat: u32,
    _7: [u8; 0x96],
    _desc_size: u16,
    _8: u32,
    _first_meta_bg: u32,
}

impl SuperBlock {
    /// The blocksize in bytes.
    pub fn block_size(&self) -> u64 {
        1 << (10 + self.log_block_size as u64)
    }

    /// The inode_size in bytes.
    pub fn inode_size(&self) -> u64 {
        if self.feature_ro_compat & 0x40 == 0 {
            128
        } else {
            self._inode_size as u64
        }
    }

    /// The group descriptor size.
    pub fn desc_size(&self) -> u64 {
        // 64-bit feature
        if self.feature_incompat & 0x80 == 0 {
            32
        } else {
            self._desc_size as u64
        }
    }

    pub fn first_meta_bg(&self) -> u64 {
        // meta_bg feature
        if self.feature_incompat & 0x10 == 0 {
            !0
        } else {
            self._first_meta_bg as u64
        }
    }

    // Calculate the disk offset of the group descriptor in bytes.
    pub fn group_desc_offset(&self, group: u64) -> u64 {
        let desc_per_block = self.block_size() / self.desc_size();
        let mut block = group / desc_per_block;
        if block < self.first_meta_bg() {
            block += self.first_block as u64 + 1;
        } else {
            let offset =
                // check for sparse_super
                if (self.feature_ro_compat & 1) == 0 || group < 2 || is_power(group, 3) || is_power(group, 5) || is_power(group, 7) {
                    1
                }
                else {
                    0
                };
            block = group * self.blocks_per_group as u64 + self.first_block as u64 + offset
        }
        block * self.block_size() + (group % desc_per_block) * self.desc_size()
    }
}

/// checks if v is a power of p
fn is_power(mut v: u64, p: u64) -> bool {
    while v > p && v % p == 0 {
        v /= p
    }
    v == p
}
