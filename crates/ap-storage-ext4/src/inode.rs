//! Sparse inode definition.

use ap_storage::meta::FileType;

/// Sparse inode definition.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Inode {
    mode: u16,
    uid_lo: u16,
    size_lo: u32,
    atime: u32,
    mtime: u32,
    ctime: u32,
    dtime: u32,
    gid_lo: u16,
    nlinks: u16,
    blocks_lo: u32,
    flags: u32,
    version_lo: u32,
    pub blocks: [u32; 15],
    generation: u32,
    xattr_lo: u32,
    size_hi: u32,
    _4: u32,
    blocks_hi: u16,
    xattr_hi: u16,
    uid_hi: u16,
    gid_hi: u16,
    _5: [u16; 2],
    extra_size: u16,
    _6: u16,
    ctime_extra: u32,
    mtime_extra: u32,
    atime_extra: u32,
    crtime: u32,
    crtime_extra: u32,
    version_hi: u32,
    _projid: u32,
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

    /// The size of the file.
    pub fn size(&self, sb_feature_incompat: u32) -> u64 {
        // this is the ext2 heuristic
        if sb_feature_incompat & 0x4000 != 0 || self.ftype() == FileType::File {
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

    pub fn mode(&self) -> u16 {
        self.mode
    }

    pub fn nlinks(&self) -> u16 {
        self.nlinks
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn uid(&self) -> u32 {
        (self.uid_hi as u32) << 16 | self.uid_lo as u32
    }

    pub fn gid(&self) -> u32 {
        (self.gid_hi as u32) << 16 | self.gid_lo as u32
    }

    pub fn version(&self) -> u64 {
        (self.version_hi as u64) << 32 | self.version_lo as u64
    }

    pub fn xattr(&self) -> u64 {
        (self.xattr_hi as u64) << 32 | self.xattr_lo as u64
    }

    /// Convert the inode timestampt to nanoseconds since epoch
    fn time_to_ts(lo: u32, hi: u32) -> i64 {
        (((hi as i64 & 3) << 30 | lo as i64) * 1_000_000_000) + (hi as i64 >> 2)
    }

    pub fn mtime(&self) -> i64 {
        Self::time_to_ts(self.mtime, self.mtime_extra)
    }

    pub fn atime(&self) -> i64 {
        Self::time_to_ts(self.atime, self.atime_extra)
    }

    pub fn crtime(&self) -> i64 {
        Self::time_to_ts(self.crtime, self.crtime_extra)
    }

    pub fn ctime(&self) -> i64 {
        Self::time_to_ts(self.ctime, self.ctime_extra)
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
