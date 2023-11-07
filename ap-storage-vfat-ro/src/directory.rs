//! Directory iteration for vfat.

use super::{File, Offset, Error, structs};
use ap_storage::{Read, ReadExt, FileType};




#[derive(Debug)]
pub struct DirEntry {
    pub nlen: usize,
    pub offset: u64,
    pub typ: FileType,
}


pub struct DirIterator<'a> {
    file: &'a File<'a>,
    offset: Offset,
}

impl<'a> DirIterator<'a> {
    pub fn new(file: &'a File) -> Self {
        Self { file, offset: 0 }
    }

    pub fn next(&mut self, name: &mut [u8; 255]) -> Result<DirEntry, Error> {
        let offset = self.offset;
        let entry: structs::DirEntry = (self.file as &dyn Read).read_object(offset)?;
        self.offset += core::mem::size_of::<structs::DirEntry>() as Offset;
        if entry.name[0] == 0 {
            return Err(anyhow::anyhow!("eof"));
        }

        let typ =
            if entry.is_dir() {
                FileType::Directory
            } else if entry.attr & 0x8 != 0 {
                FileType::Unknown
            } else {
                FileType::File
            };
                
            
                
        
        let shortname = entry.name();
        let nlen = shortname.len();
        name[..nlen].copy_from_slice(&shortname);
        Ok(DirEntry { nlen, offset, typ } )
    }
}

