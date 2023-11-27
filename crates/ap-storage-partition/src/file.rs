//! File implementation for partitions.

use crate::{attr::Attr, dir::PartitionDir, Partition};
use ap_storage::{file::File, file::FileType, Error, Offset, Read, ReadExt};

pub struct PartitionFile<'a> {
    pub(crate) disk: &'a dyn Read,
    // the absolute offset of the partition entry
    pub(crate) id: Offset,
    // the absolute offset on the disk
    pub(crate) offset: Offset,
    // the len in bytes
    pub(crate) len: Offset,
    pub(crate) drive: u8,
    pub(crate) typ: u8,
}

impl<'a> PartitionFile<'a> {
    /// Check wether this File could contain other partitions as well.
    pub fn is_dir(&self) -> bool {
        let buf: [u8; 2] = self.disk.read_object(self.offset + 0x1fe).unwrap_or([0, 0]);
        buf[0] == 0x55 && buf[1] == 0xaa
    }

    /// Return the filetype.
    pub fn ftype(&self) -> FileType {
        if self.typ == 0 || self.len == 0 {
            FileType::Unknown
        } else if self.is_dir() {
            FileType::Directory
        } else {
            FileType::File
        }
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
    type AttrType<'c> = Attr<'c> where Self: 'c;
    fn attr(&self) -> Self::AttrType<'_> {
        crate::attr::Attr { file: self }
    }

    type DirType<'a> = PartitionDir<'a> where Self: 'a;
    fn dir(&self) -> Option<<Self as ap_storage::file::File>::DirType<'_>> {
        if self.is_dir() {
            return Some(PartitionDir::new(self));
        }
        None
    }

    fn open(&self, offset: u64) -> Result<Self, Error> {
        let part = offset;
        if part > 4 {
            return Err(Error::msg("invalid number"));
        }
        let ofs = self.offset + 0x1be + part * 0x10;
        let partition: Partition = self.disk.read_object(ofs)?;
        let offset = self.offset + (partition.lba as u64) * 512;
        let len = if offset >= self.offset + self.len {
            0
        } else {
            core::cmp::min(self.offset + self.len - offset, (partition.size as u64) * 512)
        };

        Ok(PartitionFile {
            disk: self.disk,
            offset,
            len,
            id: ofs,
            typ: partition.typ,
            drive: partition.drive,
        })
    }
}
