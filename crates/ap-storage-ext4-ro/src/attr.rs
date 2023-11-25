//! File attributes for ext4.

use crate::{file::Ext4File, Error};
use ap_storage::attr;
use ap_util_slice_writer::*;

pub struct Attr<'a> {
    file: &'a Ext4File<'a>,
    offset: usize,
}

impl<'a> Attr<'a> {
    pub(crate) fn new(file: &'a Ext4File<'a>) -> Self {
        Self { file, offset: 0 }
    }
}

impl<'a> attr::Attributes for Attr<'a> {
    fn next(&mut self, name: &mut [u8], value: &mut [u8]) -> Result<Option<attr::Entry>, Error> {
        let mut name = SliceWriter(name, 0);
        let mut value = SliceWriter(value, 0);

        macro_rules! get {
            ($name:ident, $value:expr) => {{
                core::write!(&mut name, stringify!($name))?;
                core::write!(&mut value, "{:x}", $value)?;
            }};
        }
        match self.offset {
            0 => get!(blocks, self.file.inode.blocks(self.file.fs.sb.block_size())),
            1 => get!(nlinks, self.file.inode.nlinks()),
            2 => get!(mode, self.file.inode.mode),
            3 => get!(version, self.file.inode.version),
            4 => get!(btime, self.file.inode.btime()),
            5 => get!(atime, self.file.inode.atime()),
            _ => return Ok(None),
        }

        self.offset += 1;
        Ok(Some(attr::Entry {
            name_len: name.1,
            value_len: value.1,
        }))
    }
}
