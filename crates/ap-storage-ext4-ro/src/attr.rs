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
            ($name:ident) => {{
                core::write!(&mut name, stringify!($name))?;
                core::write!(&mut value, "{:x}", self.file.inode.$name())?;
            }};
        }
        match self.offset {
            0 => {
                core::write!(&mut name, "blocks")?;
                core::write!(&mut value, "{:x}", self.file.inode.blocks(self.file.fs.sb.block_size()))?;
            }
            1 => get!(nlinks),
            2 => get!(mode),
            3 => get!(flags),
            4 => get!(uid),
            5 => get!(gid),
            6 => get!(version),
            7 => get!(atime),
            8 => get!(crtime),
            9 => get!(ctime),
            10 => get!(xattr),
            11 => get!(generation),
            _ => return Ok(None),
        }

        self.offset += 1;
        Ok(Some(attr::Entry {
            name_len: name.1,
            value_len: value.1,
        }))
    }
}
