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

/// A sorted list of fields.
type AttrType = (
    &'static str,
    fn(&mut SliceWriter, &File) -> Result<(), core::fmt::Error>,
);
const ATTRS: [AttrType; 6] = [
    ("atime", |buf, file| core::write!(buf, "{:x}", file.inode.mtime())),
    ("attr", |buf, file| core::write!(buf, "{:x}", file.inode.attr() as u64)),
    ("btime", |buf, file| core::write!(buf, "{:x}", file.inode.mtime())),
    ("id", |buf, file| core::write!(buf, "{:x}", file.id)),
    ("mtime", |buf, file| core::write!(buf, "{:x}", file.inode.mtime())),
    ("size", |buf, file| core::write!(buf, "{:x}", file.size())),
];

impl<'a> attr::Attributes for Attr<'a> {
    fn next(&mut self, name: &mut [u8], value: &mut [u8]) -> Result<Option<attr::Entry>, Error> {
        let Some((n, f)) = ATTRS.get(self.offset) else {
            return Ok(None);
        };

        let mut name = SliceWriter(name, 0);
        let mut value = SliceWriter(value, 0);

        core::write!(&mut name, "{}", n)?;
        f(&mut value, self.file)?;

        self.offset += 1;
        Ok(Some(attr::Entry {
            name_len: name.1,
            value_len: value.1,
        }))
    }
}
