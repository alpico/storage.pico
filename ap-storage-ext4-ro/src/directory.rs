//! Directory iterator.
use super::{Error, FileType, Read, ReadExt};
use ap_storage::directory::{self, Iterator};

/// A directory iterator.
pub struct DirIterator<'a> {
    parent: &'a dyn Read,
    offset: u64,
}

impl<'a> DirIterator<'a> {
    pub fn new(parent: &'a dyn Read) -> Self {
        Self { parent, offset: 0 }
    }
}

impl<'a> Iterator for DirIterator<'a> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<directory::Item>, Error> {
        const O: usize = core::mem::size_of::<DirEntryHeader>();

        let header: DirEntryHeader = match self.parent.read_object(self.offset) {
            Ok(x) => x,
            Err(x) if x.is::<ap_storage::PartialReadError>() => return Ok(None),
            Err(x) => return Err(x),
        };
        let nlen = core::cmp::min(header.name_len as usize, name.len());
        extern crate std;

        if nlen > 0 {
            let n = self
                .parent
                .read_bytes(self.offset + O as u64, &mut name[..nlen])?;
            if n < nlen {
                return Err(anyhow::anyhow!("truncated dir"));
            }
        }
        let offset = self.offset;
        self.offset += header.rec_len as u64;
        let mut typ = header.typ();
        if typ == FileType::Directory && offset <= 0x18 {
            typ = FileType::Parent;
        }

        Ok(Some(directory::Item {
            offset,
            nlen,
            typ,
            id: header.inode(),
        }))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DirEntryHeader {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
}

impl DirEntryHeader {
    pub fn typ(&self) -> FileType {
        match self.file_type {
            1 => FileType::File,
            2 => FileType::Directory,
            7 => FileType::SymLink,
            _ => FileType::Unknown,
        }
    }
    pub fn inode(&self) -> u64 {
        self.inode as u64
    }
}
