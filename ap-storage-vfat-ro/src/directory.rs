//! Directory iteration for vfat.

use super::{DirectoryEntry, Error, File, Offset};
use ap_storage::{FileType, Read, ReadExt};

#[derive(Debug)]
pub struct DirEntry {
    /// The offset inside the parent. This is used to open the file relative to the parent.
    pub offset: u64,
    /// An unique ID of the referenced file.  Usefull for detecting hard-links.
    pub id: u64,
    /// The maximal length of the name this file has.
    pub nlen: usize,
    /// The file-type.
    pub typ: FileType,
}

pub struct DirIterator<'a> {
    file: &'a File<'a>,
    offset: Offset,
}

impl<'a> DirIterator<'a> {
    pub fn new(file: &'a File, skip_ptr: bool) -> Self {
        Self {
            file,
            offset: if skip_ptr { 2 } else { 0 },
        }
    }

    pub fn next(&mut self, name: &mut [u8]) -> Result<Option<DirEntry>, Error> {
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
        Ok(Some(DirEntry {
            offset: self.offset - 1,
            nlen,
            typ,
            id: entry.cluster() as u64,
        }))
    }
}
