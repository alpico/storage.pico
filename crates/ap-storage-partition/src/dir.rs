//! Directory emulation.

use crate::{file::PartitionFile, Partition};
use ap_storage::{
    directory::{DirEntry, DirIterator},
    file::{File, FileType},
    Error, ReadExt,
};
use ap_util_slice_writer::*;

pub enum Alias {
    Raw,
    Boot,
    Type,
    Max,
}

pub struct PartitionDir<'a> {
    file: &'a PartitionFile<'a>,
    pos: usize,
}

impl<'a> PartitionDir<'a> {
    pub fn new(file: &'a PartitionFile<'a>) -> Self {
        Self { file, pos: 0 }
    }
}

impl DirIterator for PartitionDir<'_> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<DirEntry>, Error> {
        if self.pos >= 4 * Alias::Max as usize {
            return Ok(None);
        }

        let part = self.pos / Alias::Max as usize;
        let partition: Partition = self
            .file
            .disk
            .read_object(self.file.offset + 0x1be + part as u64 * 0x10)?;
        let mut writer = SliceWriter(name, 0);
        match self.pos % Alias::Max as usize {
            x if x == Alias::Raw as usize => core::write!(&mut writer, "raw-{}", part).unwrap(),
            x if x == Alias::Boot as usize => {
                if partition.drive & 0x80 != 0 {
                    core::write!(&mut writer, "boot-{}", part).unwrap();
                }
            }
            x if x == Alias::Type as usize => {
                core::write!(&mut writer, "type-{:02x}-{}", partition.typ, part).unwrap();
            }
            _ => {
                unreachable!()
            }
        };

        let typ = if partition.typ == 0 || partition.size == 0 || writer.1 == 0 {
            FileType::Unknown
        } else if let Ok(child) = self.file.open(self.pos as u64) {
            if child.is_dir() {
                FileType::Directory
            } else {
                FileType::File
            }
        } else {
            FileType::Unknown
        };
        self.pos += 1;
        Ok(Some(DirEntry {
            nlen: writer.1,
            id: part as u64,
            offset: self.pos as u64 - 1,
            typ,
        }))
    }
}
