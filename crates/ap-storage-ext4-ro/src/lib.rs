//! Read-only access to the ext{2,3,4} filesystems.
//!
//! ## Features
//!
//! - `file_extents` - enable extents in files that were introduced with ext4.
//! - `file_blocks`  - enable legacy blocks in files.

#![no_std]

pub(crate) mod block;
mod dir;
pub(crate) mod extent;
pub mod file;

use dir::Dir;

use ap_storage::{meta::FileType, Error, FileSystem, Offset, Read, ReadExt};
use ap_storage_ext4::{inode::Inode, superblock::SuperBlock};

#[derive(Clone)]
pub struct Ext4Fs<'a> {
    disk: &'a dyn Read,
    sb: SuperBlock,
    leaf_optimization: bool,
}

impl<'a> Ext4Fs<'a> {
    /// Mount the filesystem..
    pub fn new(disk: &'a dyn Read, leaf_optimization: bool) -> Result<Ext4Fs<'a>, Error> {
        let sb = disk.read_object::<SuperBlock>(0x400)?;

        // check the magic
        if sb.magic != 0xef53 {
            return Err(anyhow::anyhow!("not an ext2,3,4 filesystem"));
        }

        // support FILETYPE, META_BG, EXTENTS, 64BIT and ignore RECOVER, JOURNAL_DEV, FLEX_BG
        let feature_incompat = if cfg!(feature = "file_extents") { 0xd2 } else { 0x92 };
        if sb.feature_incompat & !(feature_incompat | 0x20c) != 0 {
            return Err(anyhow::anyhow!("incompatible features {:x}", sb.feature_incompat));
        }
        Ok(Self {
            disk,
            sb,
            leaf_optimization,
        })
    }

    /// Read an inode.
    pub fn inode(&self, nr: u64) -> Result<Inode, Error> {
        if nr > self.sb.inode_count as u64 {
            return Err(anyhow::anyhow!("no such inode"));
        }

        // inode numbers start at one
        let nr = nr - 1;

        let group = nr / self.sb.inodes_per_group as u64;
        let nr = nr % self.sb.inodes_per_group as u64;

        // the offset inside the inode table
        let inode_ofs = nr * self.sb.inode_size();

        let group_desc_offset = self.sb.group_desc_offset(group);

        // get the inode block from the descriptor table.
        let inode_block = {
            let lo = self.disk.read_object::<u32>(group_desc_offset + 0x8)?;
            let hi = {
                if self.sb.desc_size() >= 64 {
                    self.disk.read_object::<u32>(group_desc_offset + 0x28)?
                } else {
                    0
                }
            };
            ((hi as u64) << 32) | lo as u64
        };

        // The inode might be smaller on disk due to backward compatiblity.
        let mut buf = [0u8; core::mem::size_of::<Inode>()];
        let n = core::cmp::min(core::mem::size_of::<Inode>(), self.sb.inode_size() as usize);
        self.disk
            .read_exact(inode_block * self.sb.block_size() + inode_ofs, &mut buf[..n])?;
        Ok(unsafe { core::mem::transmute(buf) })
    }
}

impl<'a> FileSystem<'a> for Ext4Fs<'a> {
    type FileType = file::Ext4File<'a>;
    fn root(&'a self) -> Result<Self::FileType, Error> {
        file::Ext4File::new(self, 2)
    }
}
