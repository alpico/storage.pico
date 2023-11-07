//! Directory iteration for vfat.

use super::{DirectoryEntry, Error, FatFile, Offset};
use ap_storage::{file::FileType, Read, ReadExt, directory::{self, Iterator}};

pub struct DirIterator<'a> {
    file: &'a FatFile<'a>,
    offset: Offset,
}

impl<'a> DirIterator<'a> {
    pub fn new(file: &'a FatFile) -> Self {
        Self {
            file,
            offset: 0,
        }
    }
}

impl<'a> Iterator for DirIterator<'a> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<directory::Item>, Error> {
        let entry: DirectoryEntry = if !self.file.is_root() {
            (self.file as &dyn Read).read_object(self.offset * 32)?
        } else {
            // the root directory does not have self-pointers - fabricate them
            match self.offset {
                0 => self.file.inode,
                1 => self.file.inode,
                _ => (self.file as &dyn Read).read_object(self.offset * 32 - 64)?,
            }
        };

        if entry.name[0] == 0 {
            return Ok(None);
        }
        let typ = if entry.attr & 0x8 != 0 || entry.name[0] == 0xe5 {
            FileType::Unknown
        } else if self.offset < 2 {
            FileType::Parent
        } else if entry.is_dir() {
            FileType::Directory
        } else {
            FileType::File
        };

        let shortname = entry.name();
        let mut nlen = shortname.trim_ascii().len();

        if self.offset == 0 && self.file.is_root() {
            // drop one dot from the first pointer
            nlen = 1;
        }
        let n = core::cmp::min(nlen, name.len());
        name[..n].copy_from_slice(&shortname[..n]);

        self.offset += 1;
        Ok(Some(directory::Item {
            offset: self.offset - 1,
            nlen,
            typ,
            id: entry.cluster() as u64,
        }))
    }
}
