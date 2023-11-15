//! File implementation for partitions.

use crate::{dir::PartitionDir, Partition};
use ap_storage::{file::File, Error, Offset, Read, ReadExt};

pub struct PartitionFile<'a> {
    pub(crate) disk: &'a dyn Read,
    // the absolute offset on the disk
    pub(crate) offset: Offset,
    // the len in bytes
    len: Offset,
}

impl<'a> PartitionFile<'a> {
    pub fn new(disk: &'a dyn Read, offset: Offset, len: Offset) -> Self {
        Self { disk, offset, len }
    }
    /// Check wether this File could contain other partitions as well.
    pub fn is_dir(&self) -> bool {
        let buf: [u8; 2] = self.disk.read_object(self.offset + 0x1fe).unwrap_or([0, 0]);
        buf[0] == 0x55 && buf[1] == 0xaa
    }
}

impl Read for PartitionFile<'_> {
    fn read_bytes(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        if offset >= self.offset + self.len {
            return Ok(0);
        }
        let maxn = core::cmp::min(self.offset + self.len - offset, buf.len() as u64) as usize;
        self.disk.read_bytes(self.offset + offset, &mut buf[..maxn])
    }
}

impl File for PartitionFile<'_> {
    type DirType<'a> = PartitionDir<'a> where Self: 'a;
    fn dir(&self) -> Option<<Self as ap_storage::file::File>::DirType<'_>> {
        if self.is_dir() {
            return Some(PartitionDir::new(self));
        }
        None
    }

    fn open(&self, offset: u64) -> Result<Self, anyhow::Error> {
        let part = offset / crate::dir::Alias::Max as u64;
        if part > 4 {
            return Err(anyhow::anyhow!("invalid number"));
        }
        let partition: Partition = self.disk.read_object(self.offset + 0x1be + part * 0x10)?;
        let offset = self.offset + (partition.lba as u64) * 512;
        let len = if offset >= self.offset + self.len {
            0
        } else {
            core::cmp::min(
                self.offset + self.len - offset,
                (partition.size as u64) * 512,
            )
        };

        Ok(PartitionFile {
            disk: self.disk,
            offset,
            len,
        })
    }
    fn size(&self) -> u64 {
        self.len
    }
    fn id(&self) -> u64 {
        self.offset
    }
}
