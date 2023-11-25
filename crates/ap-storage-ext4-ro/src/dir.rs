//! Directory iterator.
use super::{Error, FileType, Read, ReadExt};
use ap_storage::directory::{DirEntry, DirIterator};
use ap_storage_ext4::dir::DirEntryHeader;

/// A directory iterator.
pub struct Dir<'a> {
    parent: &'a dyn Read,
    offset: u64,
}

impl<'a> Dir<'a> {
    pub fn new(parent: &'a dyn Read) -> Self {
        Self { parent, offset: 0 }
    }
}

impl<'a> DirIterator for Dir<'a> {
    fn next(&mut self, name: &mut [u8]) -> Result<Option<DirEntry>, Error> {
        const O: usize = core::mem::size_of::<DirEntryHeader>();

        let header: DirEntryHeader = match self.parent.read_object(self.offset) {
            Ok(x) => x,
            Err(x) if x.is::<ap_storage::PartialReadError>() => return Ok(None),
            Err(x) => return Err(x),
        };
        let nlen = core::cmp::min(header.name_len as usize, name.len());
        extern crate std;

        if nlen > 0 {
            let n = self.parent.read_bytes(self.offset + O as u64, &mut name[..nlen])?;
            if n < nlen {
                return Err(anyhow::anyhow!("truncated dir"));
            }
        }
        let offset = self.offset;
        self.offset += header.rec_len as u64;
        let mut typ = header.typ();
        if typ == FileType::Directory && offset < 0x18 {
            typ = FileType::Parent;
        }

        Ok(Some(DirEntry {
            offset,
            nlen,
            typ,
            id: header.inode(),
        }))
    }
}
