//! Directory emulation.

use crate::{file::PartitionFile, Partition};
use ap_storage::{
    directory::{DirEntry, DirIterator},
    file::{File, FileType},
    Error, ReadExt,
};
use ap_util_slice_writer::*;

pub struct PartitionDir<'a> {
    file: &'a PartitionFile<'a>,
    pos: u64,
}

impl<'a> PartitionDir<'a> {
    pub fn new(file: &'a PartitionFile<'a>) -> Self {
        Self { file, pos: 0 }
    }
}

impl DirIterator for PartitionDir<'_> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<DirEntry>, Error> {
        if self.pos >= 4 {
            return Ok(None);
        }

        let partition: Partition = self.file.disk.read_object(self.file.offset + 0x1be + self.pos * 0x10)?;
        let mut writer = SliceWriter(name, 0);
        core::write!(&mut writer, "part-{}", self.pos)?;

        let typ = if partition.typ == 0 || partition.size == 0 || writer.1 == 0 {
            FileType::Unknown
        } else if let Ok(child) = self.file.open(self.pos) {
            child.ftype()
        } else {
            FileType::Unknown
        };
        self.pos += 1;
        Ok(Some(DirEntry {
            nlen: writer.1,
            id: self.pos - 1,
            offset: self.pos - 1,
            typ,
        }))
    }
}
