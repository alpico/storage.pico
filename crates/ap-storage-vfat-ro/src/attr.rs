//! File attributes for vfat.

use super::file::File;
use ap_storage::attr::{AttrType, Attributes};
use ap_util_slice_writer::*;

pub struct Attr<'a> {
    pub(crate) file: &'a File<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a (AttrType, &'a str);
    type IntoIter = core::slice::Iter<'a, (AttrType, &'a str)>;
    fn into_iter(self) -> Self::IntoIter {
        [
            (AttrType::Raw, "ftype"),
            (AttrType::U64, "attr"),
            (AttrType::U64, "id"),
            (AttrType::U64, "size"),
            (AttrType::I64, "atime"),
            (AttrType::I64, "btime"),
            (AttrType::I64, "mtime"),
        ]
        .iter()
    }
}

impl<'a> Attributes<'a> for Attr<'a> {
    fn get_raw(&mut self, name: &str, value: &mut [u8]) -> Option<usize> {
        let mut value = SliceWriter(value, 0);
        match name {
            "ftype" => write!(value, "{:?}", self.file.ftype()).ok()?,
            _ => return None,
        }
        Some(value.1)
    }

    fn get_u64(&mut self, name: &str) -> Option<u64> {
        Some(match name {
            "attr" => self.file.inode.attr() as u64,
            "id" => self.file.id,
            "size" => self.file.size(),
            _ => return None,
        })
    }
    fn get_i64(&mut self, name: &str) -> Option<i64> {
        Some(match name {
            "atime" => self.file.inode.atime(),
            "btime" => self.file.inode.btime(),
            "mtime" => self.file.inode.mtime(),
            _ => return None,
        })
    }
}
