//! Read-only access to the ext{2,3,4} filesystems.

#![no_std]

mod directory;
mod file;
mod inode;
mod superblock;

pub use directory::DirIterator;
pub use file::File;
pub use inode::Inode;
pub use superblock::SuperBlock;

use ap_storage::{Error, Offset, Read, ReadExt};

/// Generic file-types.
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    SymLink,
    Unknown,
}

#[derive(Clone)]
pub struct Ext4Fs<'a> {
    disk: &'a dyn Read,
    sb: SuperBlock,
    leaf_optimization: bool,
}

impl<'a> Ext4Fs<'a> {
    /// Mount the filesystem..
    pub fn mount(disk: &'a dyn Read, leaf_optimization: bool) -> Result<Ext4Fs<'a>, Error> {
        let sb = disk.read_object::<SuperBlock>(0x400)?;

        // check the magic
        if sb.magic != 0xef53 {
            return Err(anyhow::anyhow!("not an ext2,3,4 filesystem"));
        }

        // support FILETYPE, META_BG, EXTENTS, 64BIT and ignore RECOVER, JOURNAL_DEV, FLEX_BG
        if sb.feature_incompat & !(0xd2 | 0x20c) != 0 {
            return Err(anyhow::anyhow!(
                "incompatible features {:x}",
                sb.feature_incompat
            ));
        }
        Ok(Self { disk, sb, leaf_optimization })
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

        
        self.disk.read_object(inode_block * self.sb.block_size() + inode_ofs)
    }

    /// Return the root directory
    pub fn root(&self) -> Result<File, Error> {
        File::new(self, 2)
    }
}
