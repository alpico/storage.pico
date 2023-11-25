//! File attributes for vfat.

use super::{file::File, Error};
use ap_storage::attr;
use ap_util_slice_writer::*;

pub struct Attr<'a> {
    file: &'a File<'a>,
    offset: usize,
}

impl<'a> Attr<'a> {
    pub(crate) fn new(file: &'a File<'a>) -> Self {
        Self { file, offset: 0 }
    }
}

impl<'a> attr::Attributes for Attr<'a> {
    fn next(&mut self, name: &mut [u8], value: &mut [u8]) -> Result<Option<attr::Entry>, Error> {
        let mut name = SliceWriter(name, 0);
        let mut value = SliceWriter(value, 0);

        match self.offset {
            0 => {
                core::write!(&mut name, "attr")?;
                core::write!(&mut value, "{:x}", self.file.inode.attr)?;
            }
            1 => {
                core::write!(&mut name, "btime")?;
                core::write!(&mut value, "{:x}", self.file.inode.btime())?;
            }
            2 => {
                core::write!(&mut name, "atime")?;
                core::write!(&mut value, "{:x}", self.file.inode.atime())?;
            }
            _ => return Ok(None),
        }

        self.offset += 1;
        Ok(Some(attr::Entry {
            name_len: name.1,
            value_len: value.1,
        }))
    }
}
