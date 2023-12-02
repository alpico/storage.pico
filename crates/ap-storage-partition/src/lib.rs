//! MBR Partition support.
#![no_std]

use ap_storage::{Error, FileSystem, Read, ReadExt, msg2err};
mod attr;
mod dir;
mod file;

/// A file-system that makes MBR partitions available as files.
#[derive(Clone)]
pub struct PartitionFS<'a> {
    disk: &'a dyn Read,
    len: u64,
}

/// A single partition - on-disk format.
#[repr(C)]
#[derive(Clone)]
pub struct Partition {
    /// The BIOS drive number including the BOOT bit.
    pub drive: u8,
    /// The obsolete CHS values for starting the partition.
    pub _chs_start: [u8; 3],
    /// The partition type.
    pub typ: u8,
    /// The obsolete CHS values for ending the partition.
    pub _chs_end: [u8; 3],
    /// The logical-block address for the start of the partition.
    pub lba: u32,
    /// The size of the partition in blocks.
    pub size: u32,
}

impl<'a> PartitionFS<'a> {
    /// Mount the filesystem.
    pub fn new(disk: &'a dyn Read) -> Result<Self, Error> {
        let buf: [u8; 512] = disk.read_object(0)?;
        if buf[0x1fe] != 0x55 || buf[0x1ff] != 0xaa {
            return Err(msg2err!("not an MBR"));
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
    fn root(&'a self) -> Result<<Self as FileSystem<'a>>::FileType, Error> {
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
