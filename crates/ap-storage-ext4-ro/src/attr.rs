//! File attributes for ext4.
use crate::file::Ext4File;
use ap_storage::attr::{AttrType, Attributes};
use ap_util_slice_writer::*;

pub struct Attr<'a> {
    pub(crate) file: &'a Ext4File<'a>,
}

impl<'a> IntoIterator for Attr<'a> {
    type Item = &'a (AttrType, &'a str);
    type IntoIter = core::slice::Iter<'a, (AttrType, &'a str)>;
    fn into_iter(self) -> Self::IntoIter {
        [
            (AttrType::Raw, "ftype"),
            (AttrType::U64, "blocks"),
            (AttrType::U64, "flags"),
            (AttrType::U64, "generation"),
            (AttrType::U64, "gid"),
            (AttrType::U64, "id"),
            (AttrType::U64, "mode"),
            (AttrType::U64, "nlinks"),
            (AttrType::U64, "size"),
            (AttrType::U64, "uid"),
            (AttrType::U64, "version"),
            (AttrType::U64, "xattr"),
            (AttrType::I64, "atime"),
            (AttrType::I64, "crtime"),
            (AttrType::I64, "ctime"),
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
            "blocks" => self.file.inode.blocks(self.file.fs.sb.block_size()),
            "flags" => self.file.inode.flags().into(),
            "generation" => self.file.inode.generation().into(),
            "gid" => self.file.inode.gid().into(),
            "id" => self.file.nr,
            "mode" => self.file.inode.mode().into(),
            "nlinks" => self.file.inode.nlinks().into(),
            "size" => self.file.inode.size(self.file.fs.sb.feature_incompat),
            "uid" => self.file.inode.uid().into(),
            "version" => self.file.inode.version(),
            "xattr" => self.file.inode.xattr(),
            _ => return None,
        })
    }
    fn get_i64(&mut self, name: &str) -> Option<i64> {
        Some(match name {
            "atime" => self.file.inode.atime(),
            "crtime" => self.file.inode.crtime(),
            "ctime" => self.file.inode.ctime(),
            "mtime" => self.file.inode.mtime(),
            _ => return None,
        })
    }
}
