//! File attributes for vfat.

use super::file::File;
use ap_storage::attr::{self, new_attr, Attributes, Value};
use ap_util_slice_writer::*;

new_attr!(ATTR, U64, "File attribute bits.");

pub struct Attr<'a> {
    pub(crate) file: &'a File<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a &'a str;
    type IntoIter = core::slice::Iter<'a, &'a str>;
    fn into_iter(self) -> Self::IntoIter {
        [
            ATTR,
            attr::ATIME,
            attr::BTIME,
            attr::FTYPE,
            attr::ID,
            attr::MTIME,
            attr::SIZE,
        ]
        .iter()
    }
}

impl<'a> Attributes<'a> for Attr<'a> {
    fn get(&self, name: &str, buf: &mut [u8]) -> Option<Value> {
        Some(match name {
            ATTR => self.file.inode.attr().into(),
            attr::FTYPE => {
                let mut value = SliceWriter(buf, 0);
                write!(value, "{:?}", self.file.ftype()).ok()?;
                Value::Raw(value.1)
            }
            attr::ID => self.file.id.into(),
            attr::SIZE => self.file.size().into(),
            attr::ATIME => self.file.inode.atime().into(),
            attr::BTIME => self.file.inode.btime().into(),
            attr::MTIME => self.file.inode.mtime().into(),
            _ => return None,
        })
    }
}
