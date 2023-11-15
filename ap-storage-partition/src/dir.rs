//! Directory emulation.

use crate::{file::PartitionFile, Partition};
use ap_storage::{
    directory::{Item, Iterator},
    file::{File, FileType},
    Error, ReadExt,
};
use core::fmt::Write;

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

impl Iterator for PartitionDir<'_> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<Item>, Error> {
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
        Ok(Some(Item {
            nlen: writer.1,
            id: part as u64,
            offset: self.pos as u64 - 1,
            typ,
        }))
    }
}

/// Write into a slice of bytes while truncating on overflow.
struct SliceWriter<'a>(pub &'a mut [u8], pub usize);
impl Write for SliceWriter<'_> {
    fn write_str(&mut self, value: &str) -> Result<(), core::fmt::Error> {
        let b = value.as_bytes();
        if self.1 < self.0.len() {
            let n = core::cmp::min(self.0.len() - self.1, b.len());
            self.0[self.1..self.1 + n].copy_from_slice(&b[..n]);
        }
        self.1 += b.len();
        Ok(())
    }
}