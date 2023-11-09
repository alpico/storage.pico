//! Sparse inode definition.

use ap_storage::file::FileType;

/// Sparse inode definition.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Inode {
    mode: u16,
    _0: u16,
    size_lo: u32,
    _1: [u32; 4],
    _2: u16,
    nlinks: u16,
    blocks_lo: u32,
    flags: u32,
    pub version: u32,
    pub blocks: [u32; 15],
    _3: [u32; 2],
    size_hi: u32,
    _4: u32,
    blocks_hi: u16,
}

impl Inode {
    /// The file type.
    pub fn ftype(&self) -> FileType {
        match self.mode >> 12 {
            0x8 => FileType::File,
            0x4 => FileType::Directory,
            0xa => FileType::SymLink,
            _ => FileType::Unknown,
        }
    }

    /// The size.
    pub fn size(&self) -> u64 {
        // this is the ext2 heuristic
        if self.ftype() == FileType::File {
            ((self.size_hi as u64) << 32) | self.size_lo as u64
        } else {
            self.size_lo as u64
        }
    }

    /// The extent.
    pub fn extent(&self) -> Option<&[u32]> {
        if self.flags & 0x80000 == 0 {
            return None;
        }
        let data = &self.blocks;
        Some(data)
    }

    pub fn nlinks(&self) -> u16 {
        self.nlinks
    }

    /// Get the number of blocks.
    pub fn blocks(&self, sb_block_size: u64) -> u64 {
        let mut res = (self.blocks_lo as u64) | ((self.blocks_hi as u64) << 32);
        // EXT4_HUGE_FILE_FL is set
        if self.flags & 0x40000 != 0 {
            res *= sb_block_size / 512;
        }
        res
    }
}