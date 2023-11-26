//! MBR Partition support.
#![no_std]

use ap_storage::{Error, FileSystem, Read, ReadExt};
mod attr;
mod dir;
mod file;

#[derive(Clone)]
pub struct PartitionFS<'a> {
    disk: &'a dyn Read,
    len: u64,
}

/// A single partition - on-disk format.
#[repr(C)]
#[derive(Clone)]
pub struct Partition {
    pub drive: u8,
    pub _chs_start: [u8; 3],
    pub typ: u8,
    pub _chs_end: [u8; 3],
    pub lba: u32,
    pub size: u32,
}

impl<'a> PartitionFS<'a> {
    /// Mount the filesystem.
    pub fn new(disk: &'a dyn Read) -> Result<Self, Error> {
        let buf: [u8; 512] = disk.read_object(0)?;
        if buf[0x1fe] != 0x55 || buf[0x1ff] != 0xaa {
            return Err(anyhow::anyhow!("not an MBR"));
        }
        // find the maximum length all partitions occupy
        let primary: [Partition; 4] = unsafe { core::ptr::read_unaligned(buf.as_ptr().add(0x1be).cast()) };
        let len = primary.iter().map(|x| x.lba + x.size).fold(0, core::cmp::max);
        Ok(Self {
            disk,
            len: (len as u64) * 512,
        })
    }
}

impl<'a> FileSystem<'a> for PartitionFS<'a> {
    type FileType = file::PartitionFile<'a>;
    fn root(&'a self) -> Result<<Self as FileSystem<'a>>::FileType, anyhow::Error> {
        Ok(file::PartitionFile {
            disk: self.disk,
            offset: 0,
            len: self.len,
            id: 0,
            typ: 0,
            drive: 0x80,
        })
    }
}
